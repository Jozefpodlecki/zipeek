use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};
use zipeek_ai_client::{AiClient, StreamUpdate, TranslationDirection};
use zipeek_ai_protocol::sentence_analysis;

#[derive(Deserialize)]
struct SentenceFixture {
    direction: String,
    source: String,
    plain: String,
    analysis: Value,
}

fn fixture(name: &str) -> SentenceFixture {
    let raw = match name {
        "zh-en" => include_str!("fixtures/complex_zh_en.json"),
        "en-zh" => include_str!("fixtures/complex_en_zh.json"),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_str(raw).unwrap()
}

struct Reply {
    status: u16,
    payload: String,
}

fn completion(content: &str) -> Reply {
    Reply {
        status: 200,
        payload: json!({"choices":[{"message":{"content":content}}]}).to_string(),
    }
}

fn fixture_server(
    responses: Vec<Reply>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (sender, receiver) = mpsc::channel();
    let task = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            let request = read_request(&mut stream);
            sender.send(request).unwrap();
            let reason = if response.status == 200 {
                "OK"
            } else {
                "Error"
            };
            let headers = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response.status,
                reason,
                response.payload.len()
            );
            stream.write_all(headers.as_bytes()).unwrap();
            stream.write_all(response.payload.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });
    (format!("http://{addr}/v1/"), receiver, task)
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut collected = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let n = stream.read(&mut chunk).unwrap();
        assert!(n > 0, "request prematurely closed");
        collected.extend_from_slice(&chunk[..n]);
        let Some(header_end) = collected
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
        else {
            continue;
        };
        let header_end = header_end + 4;
        let headers = String::from_utf8_lossy(&collected[..header_end]);
        let length: usize = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("content-length:")
                    .or_else(|| line.strip_prefix("Content-Length:"))
            })
            .unwrap_or("0")
            .trim()
            .parse()
            .unwrap();
        if collected.len() >= header_end + length {
            return String::from_utf8(collected).unwrap();
        }
    }
}

fn body(request: &str) -> Value {
    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[tokio::test]
async fn complex_chinese_sentence_translation_and_analysis_match_contract() {
    let f = fixture("zh-en");
    let analysis_json = f.analysis.to_string();
    let (url, requests, handle) =
        fixture_server(vec![completion(&analysis_json), completion(&f.plain)]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let messages = sentence_analysis::messages(&f.source, &f.direction);
    let raw = client.complete(messages, "test-only-key").await.unwrap();
    let analysis = sentence_analysis::parse(&raw, &f.source, &f.direction).unwrap();
    assert_eq!(analysis.translation, f.plain);
    assert_eq!(analysis.grammar.len(), 2);
    assert_eq!(analysis.vocabulary.len(), 3);
    let plain = client
        .translate(&f.source, TranslationDirection::ChineseToEnglish, "")
        .await
        .unwrap();
    assert_eq!(plain, f.plain);
    let request = requests.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(
        request
            .to_ascii_lowercase()
            .contains("authorization: bearer test-only-key")
    );
    assert_eq!(body(&request)["model"], "fixture-model");
    assert_eq!(body(&request)["stream"], true);
    assert_eq!(body(&request)["messages"][1]["content"], f.source);
    assert!(
        body(&request)["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("vocabulary")
    );
    handle.join().unwrap();
}

#[tokio::test]
async fn complex_english_sentence_translation_and_analysis_match_contract() {
    let f = fixture("en-zh");
    let analysis_json = f.analysis.to_string();
    let (url, requests, handle) =
        fixture_server(vec![completion(&analysis_json), completion(&f.plain)]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let raw = client
        .complete(sentence_analysis::messages(&f.source, &f.direction), "")
        .await
        .unwrap();
    let analysis = sentence_analysis::parse(&raw, &f.source, &f.direction).unwrap();
    assert_eq!(analysis.translation, f.plain);
    assert_eq!(analysis.grammar.len(), 2);
    assert_eq!(analysis.vocabulary.len(), 3);
    assert_eq!(
        client
            .translate(&f.source, TranslationDirection::EnglishToChinese, "")
            .await
            .unwrap(),
        f.plain
    );
    let request = requests.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(body(&request)["messages"][1]["content"], f.source);
    handle.join().unwrap();
}

#[tokio::test]
async fn provider_rejection_exposes_safe_diagnostics_and_allows_translation_retry() {
    let reply = Reply {
        status: 400,
        payload: json!({"error": {
            "code": "context_length_exceeded",
            "message": "Bearer sk-secret-12345678 exceeded context length"
        }})
        .to_string(),
    };
    let (url, requests, handle) =
        fixture_server(vec![reply, completion("尽管审核通过，仍需谨慎行事。")]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let result = client
        .complete(
            vec![json!({"role":"user", "content":"Analyse this sentence"})],
            "sk-secret-12345678",
        )
        .await;
    let error = result.unwrap_err().to_string();
    assert!(error.contains("HTTP 400"));
    assert!(error.contains("context_length_exceeded"));
    assert!(!error.contains("sk-secret-12345678"));
    assert_eq!(
        client
            .translate(
                "Although the review passed, caution remains necessary.",
                TranslationDirection::EnglishToChinese,
                ""
            )
            .await
            .unwrap(),
        "尽管审核通过，仍需谨慎行事。"
    );
    requests.recv_timeout(Duration::from_secs(2)).unwrap();
    requests.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.join().unwrap();
}

#[tokio::test]
async fn rejects_non_chat_responses_and_unsafe_key_endpoint() {
    assert!(AiClient::new("ftp://example.test/v1/", "model").is_err());
    let client = AiClient::new("http://example.test/v1/", "model").unwrap();
    assert!(
        client
            .complete(vec![json!({"role":"user","content":"hi"})], "secret")
            .await
            .unwrap_err()
            .to_string()
            .contains("unencrypted")
    );
    let (url, requests, handle) = fixture_server(vec![completion("   ")]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let error = client
        .translate("Hello", TranslationDirection::EnglishToChinese, "")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("no text response"));
    requests.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.join().unwrap();
}

#[tokio::test]
async fn status_code_diagnostics_are_distinct_and_never_render_html() {
    let statuses = [401, 403, 404, 422, 429, 503];
    let replies = statuses
        .iter()
        .map(|status| Reply {
            status: *status,
            payload: "<!DOCTYPE html><html>private token</html>".to_owned(),
        })
        .collect();
    let (url, requests, handle) = fixture_server(replies);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    for status in statuses {
        let error = client
            .complete(vec![json!({"role":"user", "content":"hello"})], "")
            .await
            .unwrap_err();
        let message = error.to_string();
        assert!(message.contains(&format!("HTTP {status}")), "{message}");
        assert!(!message.contains("private token"));
        requests.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    handle.join().unwrap();
}

#[tokio::test]
async fn redirects_are_not_accepted_as_chat_completions() {
    let (url, requests, handle) = fixture_server(vec![Reply {
        status: 302,
        payload: "{}".to_owned(),
    }]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let result = client
        .complete(vec![json!({"role":"user", "content":"hello"})], "")
        .await;
    assert!(result.unwrap_err().to_string().contains("HTTP 302"));
    requests.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.join().unwrap();
}

#[tokio::test]
async fn consumes_fragmented_sse_over_real_http_and_reports_incremental_progress() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let request = read_request(&mut socket);
        assert_eq!(body(&request)["stream"], true);
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
        for event in [
            "data: {\"choices\":[{\"delta\":{\"content\":\"尽管\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"预算有限\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"，仍继续。\"}}]}\n\n",
            "data: [DONE]\n\n",
        ] {
            for piece in event.as_bytes().chunks(3) {
                let header = format!("{:X}\r\n", piece.len());
                socket.write_all(header.as_bytes()).unwrap();
                socket.write_all(piece).unwrap();
                socket.write_all(b"\r\n").unwrap();
            }
            socket.flush().unwrap();
            thread::sleep(Duration::from_millis(25));
        }
        socket.write_all(b"0\r\n\r\n").unwrap();
    });
    let client = AiClient::new(&format!("http://{addr}/v1/"), "fixture-model").unwrap();
    let mut deltas = Vec::new();
    let output = client
        .complete_streaming(
            vec![json!({"role":"user","content":"Translate the sentence"})],
            "",
            |delta| deltas.push(delta.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(output, "尽管预算有限，仍继续。");
    assert_eq!(deltas, ["尽管", "预算有限", "，仍继续。"]);
    handle.join().unwrap();
}

#[tokio::test]
async fn rejects_sse_that_ends_without_a_completion_marker() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let _ = read_request(&mut socket);
        let payload = b"data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n";
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        );
        socket.write_all(header.as_bytes()).unwrap();
        socket.write_all(payload).unwrap();
    });
    let client = AiClient::new(&format!("http://{addr}/v1/"), "fixture-model").unwrap();
    let result = client
        .complete(vec![json!({"role":"user","content":"hello"})], "")
        .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("before completion")
    );
    handle.join().unwrap();
}

#[tokio::test]
async fn reasoning_stream_reports_activity_without_exposing_thought_text() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let request = read_request(&mut socket);
        assert_eq!(body(&request)["stream"], true);
        let events = [
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private thought\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\n\n",
            "data: [DONE]\n\n",
        ];
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
        for event in events {
            for piece in event.as_bytes().chunks(4) {
                socket
                    .write_all(format!("{:X}\r\n", piece.len()).as_bytes())
                    .unwrap();
                socket.write_all(piece).unwrap();
                socket.write_all(b"\r\n").unwrap();
            }
            socket.flush().unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        socket.write_all(b"0\r\n\r\n").unwrap();
    });
    let client = AiClient::new(&format!("http://{addr}/v1/"), "fixture-model").unwrap();
    let mut updates = Vec::new();
    let result = client
        .complete_with_updates(
            vec![json!({"role": "user", "content": "Translate"})],
            "",
            |update| updates.push(update),
        )
        .await
        .unwrap();
    assert_eq!(result, "你好");
    assert_eq!(
        updates,
        [
            StreamUpdate::Reasoning { received_chars: 15 },
            StreamUpdate::Content("你好".to_owned()),
        ]
    );
    handle.join().unwrap();
}

#[tokio::test]
async fn optional_strict_json_schema_is_serialized_in_streaming_request() {
    use zipeek_ai_client::ResponseFormat;
    let (url, requests, handle) = fixture_server(vec![completion("{\"ok\":true}")]);
    let client = AiClient::new(&url, "fixture-model").unwrap();
    let schema = ResponseFormat::strict_schema(
        "fixture_response",
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["ok"],
            "properties": {"ok": {"type": "boolean"}}
        }),
    );
    let reply = client
        .complete_with_format(
            vec![zipeek_ai_client::ChatMessage::user("Say hello")],
            "",
            Some(&schema),
            |_| {},
        )
        .await
        .unwrap();
    assert_eq!(reply, "{\"ok\":true}");
    let request = requests.recv_timeout(Duration::from_secs(2)).unwrap();
    let posted = body(&request);
    assert_eq!(posted["stream"], true);
    assert_eq!(posted["response_format"]["type"], "json_schema");
    assert_eq!(posted["response_format"]["json_schema"]["strict"], true);
    assert_eq!(
        posted["response_format"]["json_schema"]["schema"]["additionalProperties"],
        false
    );
    assert_eq!(posted["messages"][0]["role"], "user");
    handle.join().unwrap();
}
