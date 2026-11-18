use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use serde_json::{Value, json};

use super::HarnessResult;

const MAX_REQUEST_SIZE: usize = 65_536;

pub(super) struct AiFixtureServer {
    port: u16,
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl AiFixtureServer {
    pub(super) fn start() -> HarnessResult<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let running = Arc::new(AtomicBool::new(true));
        let signal = Arc::clone(&running);
        let worker = thread::Builder::new()
            .name("zipeek-ai-fixture".to_owned())
            .spawn(move || serve(listener, &signal))?;
        Ok(Self {
            port,
            running,
            worker: Some(worker),
        })
    }

    pub(super) fn url(&self) -> String {
        format!("http://127.0.0.1:{}/v1/", self.port)
    }

    pub(super) fn stop(&mut self) -> HarnessResult<()> {
        self.running.store(false, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "Rust AI fixture server thread panicked")?;
        }
        Ok(())
    }
}

impl Drop for AiFixtureServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Default)]
struct FixtureState {
    scenario: String,
    records: Vec<Value>,
}

fn serve(listener: TcpListener, running: &AtomicBool) {
    let mut state = FixtureState {
        scenario: "success".to_owned(),
        records: Vec::new(),
    };
    while running.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let result = stream
                    .set_nonblocking(false)
                    .and_then(|()| stream.set_read_timeout(Some(Duration::from_secs(10))))
                    .and_then(|()| stream.set_write_timeout(Some(Duration::from_secs(10))));
                if let Err(error) = result {
                    eprintln!("Rust AI fixture connection setup failed: {error}");
                    continue;
                }
                if let Err(error) = handle(&mut stream, &mut state) {
                    eprintln!("Rust AI fixture request failed: {error}");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15));
            }
            Err(error) => {
                eprintln!("Rust AI fixture listener failed: {error}");
                break;
            }
        }
    }
}

fn request(stream: &mut TcpStream) -> Result<(String, String, Value), String> {
    let mut buffer = Vec::new();
    let mut bytes = [0u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut bytes).map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("Browser closed the fixture connection before sending headers".to_owned());
        }
        buffer.extend_from_slice(&bytes[..count]);
        if buffer.len() > MAX_REQUEST_SIZE {
            return Err("Fixture request exceeds 64 KiB".to_owned());
        }
        if let Some(index) = buffer.windows(4).position(|part| part == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = std::str::from_utf8(&buffer[..header_end]).map_err(|error| error.to_string())?;
    let mut lines = headers.lines();
    let first = lines.next().ok_or("Missing HTTP request line")?;
    let mut request_line = first.split_whitespace();
    let method = request_line.next().ok_or("Missing HTTP method")?.to_owned();
    let path = request_line
        .next()
        .ok_or("Missing HTTP path")?
        .split('?')
        .next()
        .unwrap_or("/")
        .to_owned();
    let length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.trim().parse::<usize>())
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or(0);
    if length > MAX_REQUEST_SIZE || header_end + length > MAX_REQUEST_SIZE {
        return Err("Fixture JSON body exceeds 64 KiB".to_owned());
    }
    while buffer.len() - header_end < length {
        let count = stream.read(&mut bytes).map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("Fixture JSON body was truncated".to_owned());
        }
        buffer.extend_from_slice(&bytes[..count]);
    }
    let body = if length == 0 {
        Value::Null
    } else {
        serde_json::from_slice(&buffer[header_end..header_end + length])
            .map_err(|error| format!("Invalid fixture request JSON: {error}"))?
    };
    Ok((method, path, body))
}

fn headers(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    length: Option<usize>,
) -> std::io::Result<()> {
    let label = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let size = length
        .map(|value| format!("Content-Length: {value}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "HTTP/1.1 {status} {label}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nAccess-Control-Allow-Private-Network: true\r\nCache-Control: no-store\r\nContent-Type: {content_type}\r\nConnection: close\r\n{size}\r\n"
    )
}

fn respond(stream: &mut TcpStream, status: u16, body: Value) -> std::io::Result<()> {
    if status == 204 {
        return headers(stream, status, "application/json; charset=utf-8", Some(0));
    }
    let content = body.to_string();
    headers(
        stream,
        status,
        "application/json; charset=utf-8",
        Some(content.len()),
    )?;
    stream.write_all(content.as_bytes())
}

fn stream_completion(
    stream: &mut TcpStream,
    content: &str,
    progressive: bool,
) -> std::io::Result<()> {
    headers(stream, 200, "text/event-stream; charset=utf-8", None)?;
    if progressive {
        event(
            stream,
            &json!({"reasoning_content": "private working tokens ".repeat(10)}),
        )?;
        thread::sleep(Duration::from_millis(350));
        let mut characters = content.chars();
        loop {
            let part = characters.by_ref().take(20).collect::<String>();
            if part.is_empty() {
                break;
            }
            event(stream, &json!({"content": part}))?;
            thread::sleep(Duration::from_millis(45));
        }
    } else {
        event(stream, &json!({"content": content}))?;
    }
    stream.write_all(b"data: [DONE]\n\n")
}

fn event(stream: &mut TcpStream, delta: &Value) -> std::io::Result<()> {
    write!(
        stream,
        "data: {}\n\n",
        json!({"choices": [{"delta": delta}]})
    )?;
    stream.flush()
}

fn completion(
    stream: &mut TcpStream,
    body: &Value,
    content: &str,
    progressive: bool,
) -> std::io::Result<()> {
    if body["stream"] == true {
        stream_completion(stream, content, progressive)
    } else {
        respond(
            stream,
            200,
            json!({"choices": [{"message": {"role":"assistant", "content": content}}]}),
        )
    }
}

fn message<'a>(body: &'a Value, role: &str) -> &'a str {
    body["messages"]
        .as_array()
        .and_then(|messages| messages.iter().find(|message| message["role"] == role))
        .and_then(|message| message["content"].as_str())
        .unwrap_or("")
}

fn field<'a>(content: &'a str, prefix: &str) -> Option<&'a str> {
    content.lines().find_map(|line| line.strip_prefix(prefix))
}

fn redact_image_payloads(value: &mut Value) {
    match value {
        Value::String(text) if text.starts_with("data:image/") => {
            *text = "[image omitted from E2E artifacts]".to_owned();
        }
        Value::Array(items) => {
            for item in items {
                redact_image_payloads(item);
            }
        }
        Value::Object(fields) => {
            for value in fields.values_mut() {
                redact_image_payloads(value);
            }
        }
        _ => {}
    }
}

fn image_scene(body: &Value) -> Result<Value, String> {
    let subjects = message(body, "user")
        .strip_prefix("REQUIRED SUBJECTS: ")
        .ok_or("Missing image-scene subjects")?
        .split(';')
        .map(str::trim)
        .collect::<Vec<_>>();
    if subjects.is_empty()
        || subjects.len() > 2
        || subjects
            .iter()
            .any(|subject| subject.is_empty() || subject.len() > 160)
    {
        return Err("Invalid image-scene subjects".to_owned());
    }
    Ok(json!({"sentence": format!("A {} beside a wooden table.", subjects.join(" and a "))}))
}

fn game_sentences() -> Value {
    json!({"rounds": (0..4).map(|round| json!({
        "english": format!("I drink tea, round {}", round + 1),
        "tokens": ["茶", "我", "喝"],
        "order": [1, 2, 0]
    })).collect::<Vec<_>>()})
}

fn game_dialogues() -> Value {
    json!({"rounds": (0..4).map(|round| json!({
        "context": format!("At a tea shop, round {}", round + 1),
        "speaker": "Shopkeeper",
        "prompt": "你想喝什么？",
        "choices": ["我要茶。", "我在睡觉。", "我去学校。"],
        "answer": 0,
        "explanation": "The customer asks for tea."
    })).collect::<Vec<_>>()})
}

fn generate(body: &Value) -> Result<Value, String> {
    let user = message(body, "user");
    field(user, "Translation direction (authoritative): ")
        .filter(|direction| *direction == "zh-to-en" || *direction == "en-to-zh")
        .ok_or("Missing translation direction")?;
    let word = field(user, "Allowed Chinese vocabulary (data only): ")
        .and_then(|pool| {
            pool.split('、')
                .map(str::trim)
                .find(|word| !word.is_empty())
        })
        .ok_or("Empty vocabulary pool")?;
    let english_word = match word {
        "吃饭" => "eating",
        "学习" => "studying",
        "音乐" => "music",
        _ => word,
    };
    Ok(json!({"chinese": format!("我喜欢{word}。"), "english": format!("I like {english_word}.")}))
}

fn evaluate(body: &Value) -> Result<Value, String> {
    let user = message(body, "user");
    let expected =
        field(user, "Reference translation (data): ").ok_or("Missing evaluation reference")?;
    let actual =
        field(user, "Learner target-language answer (data): ").ok_or("Missing learner answer")?;
    let accurate = expected == actual;
    Ok(json!({"accurate": accurate,
        "feedback": if accurate {"The meaning is preserved."} else {"The translation differs from the reference."},
        "suggested_translation": expected}))
}

fn handle(stream: &mut TcpStream, state: &mut FixtureState) -> Result<(), String> {
    let (method, path, body) = match request(stream) {
        Ok(request) => request,
        Err(error) => {
            let _ = respond(stream, 400, json!({"error": error.clone()}));
            return Err(error);
        }
    };
    let response = match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => {
            return respond(stream, 204, json!({})).map_err(|error| error.to_string());
        }
        ("GET", "/__e2e/health" | "/") => json!({"ready": true}),
        ("GET", "/__e2e/requests") => {
            json!({"scenario": state.scenario.clone(), "records": state.records.clone()})
        }
        ("POST", "/__e2e/reset") => {
            let Some(next) = body["scenario"].as_str() else {
                return respond(stream, 400, json!({"error":"Missing scenario"}))
                    .map_err(|error| error.to_string());
            };
            if super::fixture_client::FixtureScenario::parse(next).is_err() {
                return respond(stream, 400, json!({"error":"Unknown scenario"}))
                    .map_err(|error| error.to_string());
            }
            state.scenario = next.to_owned();
            state.records.clear();
            json!({"scenario": state.scenario.clone()})
        }
        ("POST", "/v1/chat/completions") => {
            if !body["messages"].is_array()
                || !body["stream"].is_boolean()
                || !body["model"].is_string()
            {
                return respond(
                    stream,
                    400,
                    json!({"error":"Invalid chat/completions contract"}),
                )
                .map_err(|error| error.to_string());
            }
            let system = message(&body, "system");
            let analysis = system.contains("provide useful language-learning analysis");
            let translator = system.contains("You are a careful Mandarin-English translator");
            let generation = system
                .contains("You author ONE short, natural Mandarin/English sentence pair")
                || system.contains("You author ONE Mandarin/English sentence-translation exercise");
            let evaluation =
                system.contains("You evaluate one Mandarin/English sentence translation");
            let recognition = system.contains("Read the Chinese characters or words in the image");
            let scene = system.contains("You are composing a photographic scene for a Mandarin vocabulary recognition exercise");
            let sentence_game =
                system.contains("You create Mandarin-learning sentence ordering exercises.");
            let dialogue_game =
                system.contains("You create Mandarin-learning short conversational roleplays.");
            let kind = if recognition {
                "hanzi-recognition"
            } else if sentence_game {
                "sentence-relay"
            } else if dialogue_game {
                "dialogue-quest"
            } else if scene {
                "image-scene"
            } else if analysis {
                "translator-analysis"
            } else if translator {
                "translator-fallback"
            } else if generation {
                "generation"
            } else if evaluation {
                "evaluation"
            } else {
                return respond(stream, 400, json!({"error":"Unexpected prompt"}))
                    .map_err(|error| error.to_string());
            };
            let mut safe_messages = body["messages"].clone();
            redact_image_payloads(&mut safe_messages);
            state
                .records
                .push(json!({"kind": kind, "model": body["model"], "messages": safe_messages}));
            if state.scenario == "server-error" {
                return respond(stream, 503, json!({"error":"Fixture server failure"}))
                    .map_err(|error| error.to_string());
            }
            if analysis && state.scenario == "translator-rejected-analysis" {
                return respond(stream, 400, json!({"error": {"message":"context length exceeds fixture model limit", "code":"context_length_exceeded"}})).map_err(|error| error.to_string());
            }
            if evaluation && state.scenario == "slow-evaluation" {
                thread::sleep(Duration::from_millis(1200));
            }
            let content = if recognition {
                let has_image = body["messages"].as_array().is_some_and(|messages| messages.iter().any(|message| {
                    message["content"].as_array().is_some_and(|parts| parts.iter().any(|part| {
                        part["type"] == "image_url"
                            && part["image_url"]["url"].as_str().is_some_and(|url| url.starts_with("data:image/png;base64,") && url.len() > 100)
                    }))
                }));
                if !has_image {
                    return respond(stream, 400, json!({"error":"Recognition requires an image"}))
                        .map_err(|error| error.to_string());
                }
                json!({"candidates":["汉"]}).to_string()
            } else if (sentence_game || dialogue_game)
                && state.scenario == "malformed-generation"
            {
                "{invalid-game-json".to_owned()
            } else if sentence_game {
                game_sentences().to_string()
            } else if dialogue_game {
                game_dialogues().to_string()
            } else if scene {
                match image_scene(&body) {
                    Ok(result) => result.to_string(),
                    Err(error) => {
                        return respond(stream, 400, json!({"error": error}))
                            .map_err(|error| error.to_string());
                    }
                }
            } else if analysis || translator {
                if analysis
                    && ["translator-valid-analysis", "translator-streaming"]
                        .contains(&state.scenario.as_str())
                {
                    json!({"translation":"简非常高兴。", "alternatives": [], "features": [], "nuance": [],
                        "difficulty":"beginner", "estimated_hsk":null, "rationale":"Short present-tense sentence.",
                        "vocabulary":[], "grammar":[], "notes":[]}).to_string()
                } else {
                    "简非常高兴。".to_owned()
                }
            } else if generation && state.scenario == "malformed-generation" {
                "{bad-json".to_owned()
            } else {
                match if generation {
                    generate(&body)
                } else {
                    evaluate(&body)
                } {
                    Ok(result) => result.to_string(),
                    Err(error) => {
                        return respond(stream, 400, json!({"error": error}))
                            .map_err(|error| error.to_string());
                    }
                }
            };
            return completion(
                stream,
                &body,
                &content,
                state.scenario == "translator-streaming",
            )
            .map_err(|error| error.to_string());
        }
        _ => {
            return respond(stream, 404, json!({"error":"Not found"}))
                .map_err(|error| error.to_string());
        }
    };
    respond(stream, 200, response).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_fixture_returns_bilingual_pair_and_preserves_request_records() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, fixture.port)).unwrap();
        let body = json!({"model":"fixture", "stream":false, "messages": [
            {"role":"system", "content":"You author ONE short, natural Mandarin/English sentence pair"},
            {"role":"user", "content":"Translation direction (authoritative): zh-to-en\nAllowed Chinese vocabulary (data only): 学习、吃饭"}
        ]}).to_string();
        write!(stream, "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        assert!(answer.contains("200 OK"));
        assert!(answer.contains("\\\"chinese\\\""));
        assert!(!answer.contains("pinyin"));
        let mut records = TcpStream::connect((Ipv4Addr::LOCALHOST, fixture.port)).unwrap();
        write!(
            records,
            "GET /__e2e/requests HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut result = String::new();
        records.read_to_string(&mut result).unwrap();
        assert!(result.contains("generation"));
        fixture.stop().unwrap();
    }

    fn exchange(port: u16, method: &str, path: &str, body: Option<Value>) -> String {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let content = body.map(|value| value.to_string()).unwrap_or_default();
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{content}", content.len()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    fn fixture_messages() -> Value {
        json!({"model": "fixture", "stream": false, "messages": [
            {"role": "system", "content": "You author ONE short, natural Mandarin/English sentence pair"},
            {"role": "user", "content": "Translation direction (authoritative): en-to-zh\nAllowed Chinese vocabulary (data only): 学习、吃饭"}
        ]})
    }

    #[test]
    fn fixture_handles_headers_arriving_in_multiple_packets() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, fixture.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .write_all(b"POST /__e2e/reset HTTP/1.1\r\nHost: localhost\r\n")
            .unwrap();
        thread::sleep(Duration::from_millis(80));
        stream.write_all(b"Content-Type: application/json\r\nContent-Length: 22\r\nConnection: close\r\n\r\n").unwrap();
        thread::sleep(Duration::from_millis(80));
        stream.write_all(br#"{"scenario":"success"}"#).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.contains("200 OK"), "{response}");
        assert!(response.contains("\"scenario\":\"success\""), "{response}");
        fixture.stop().unwrap();
    }

    #[test]
    fn image_scene_fixture_preserves_all_required_subjects() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let request = json!({"model": "fixture", "stream": false, "messages": [
            {"role": "system", "content": "You are composing a photographic scene for a Mandarin vocabulary recognition exercise"},
            {"role": "user", "content": "REQUIRED SUBJECTS: cat; dog"}
        ]});
        let response = exchange(fixture.port, "POST", "/v1/chat/completions", Some(request));
        assert!(response.contains("200 OK"), "{response}");
        assert!(response.contains("cat and a dog"), "{response}");
        let records = exchange(fixture.port, "GET", "/__e2e/requests", None);
        assert!(records.contains("image-scene"));
        fixture.stop().unwrap();
    }

    #[test]
    fn image_scene_fixture_rejects_missing_or_oversized_subjects() {
        for user in [
            "REQUIRED SUBJECTS: ",
            "REQUIRED SUBJECTS: cat; dog; car",
            "missing header",
        ] {
            let request = json!({"model": "fixture", "stream": false, "messages": [
                {"role": "system", "content": "You are composing a photographic scene for a Mandarin vocabulary recognition exercise"},
                {"role": "user", "content": user}
            ]});
            let mut fixture = AiFixtureServer::start().unwrap();
            let response = exchange(fixture.port, "POST", "/v1/chat/completions", Some(request));
            assert!(response.contains("400 Bad Request"), "{response}");
            fixture.stop().unwrap();
        }
    }

    #[test]
    fn error_scenarios_reset_request_records_and_recover() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let port = fixture.port;
        let generation = fixture_messages();
        let invalid = exchange(
            port,
            "POST",
            "/__e2e/reset",
            Some(json!({"scenario":"nonexistent"})),
        );
        assert!(invalid.contains("400 Bad Request"));
        let reset = exchange(
            port,
            "POST",
            "/__e2e/reset",
            Some(json!({"scenario":"malformed-generation"})),
        );
        assert!(reset.contains("200 OK"));
        let malformed = exchange(
            port,
            "POST",
            "/v1/chat/completions",
            Some(generation.clone()),
        );
        assert!(malformed.contains("{bad-json"));
        let records = exchange(port, "GET", "/__e2e/requests", None);
        assert!(records.contains("generation"));
        assert!(records.contains("malformed-generation"));
        exchange(
            port,
            "POST",
            "/__e2e/reset",
            Some(json!({"scenario":"server-error"})),
        );
        let rejected = exchange(
            port,
            "POST",
            "/v1/chat/completions",
            Some(generation.clone()),
        );
        assert!(rejected.contains("503 Service Unavailable"));
        exchange(
            port,
            "POST",
            "/__e2e/reset",
            Some(json!({"scenario":"success"})),
        );
        let recovered = exchange(port, "POST", "/v1/chat/completions", Some(generation));
        assert!(recovered.contains("200 OK"));
        let records = exchange(port, "GET", "/__e2e/requests", None);
        assert!(records.contains("success"));
        assert!(!records.contains("server-error"));
        fixture.stop().unwrap();
    }

    #[test]
    fn deterministic_ai_games_provide_four_valid_rounds() {
        let sentence_rounds = game_sentences();
        let sentences = sentence_rounds["rounds"].as_array().unwrap();
        assert_eq!(sentences.len(), 4);
        for (position, round) in sentences.iter().enumerate() {
            assert_eq!(round["order"], json!([1, 2, 0]));
            assert_eq!(round["tokens"], json!(["茶", "我", "喝"]));
            assert!(
                round["english"]
                    .as_str()
                    .unwrap()
                    .contains(&(position + 1).to_string())
            );
        }
        let dialogues = game_dialogues();
        let rounds = dialogues["rounds"].as_array().unwrap();
        assert_eq!(rounds.len(), 4);
        for round in rounds {
            assert_eq!(round["answer"], 0);
            assert_eq!(round["choices"].as_array().unwrap().len(), 3);
        }
    }

    #[test]
    fn ai_game_fixture_accepts_actual_system_prompt_contracts() {
        let mut fixture = AiFixtureServer::start().unwrap();
        for (system, marker) in [
            (
                "You create Mandarin-learning sentence ordering exercises. Output only valid JSON",
                "I drink tea, round 1",
            ),
            (
                "You create Mandarin-learning short conversational roleplays. Output only valid JSON",
                "At a tea shop, round 1",
            ),
        ] {
            let response = exchange(
                fixture.port,
                "POST",
                "/v1/chat/completions",
                Some(json!({
                    "model": "fixture-model", "stream": false,
                    "messages": [{"role":"system", "content": system}, {"role":"user", "content": "{}"}]
                })),
            );
            assert!(response.contains("200 OK"));
            assert!(response.contains(marker));
        }
        fixture.stop().unwrap();
    }

    #[test]
    fn streaming_fixture_emits_openai_compatible_sse() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let mut generation = fixture_messages();
        generation["stream"] = Value::Bool(true);
        let streamed = exchange(
            fixture.port,
            "POST",
            "/v1/chat/completions",
            Some(generation),
        );
        assert!(streamed.contains("200 OK"));
        assert!(streamed.contains("text/event-stream"));
        assert!(streamed.contains("data: {"));
        assert!(streamed.contains("data: [DONE]"));
        fixture.stop().unwrap();
    }
    #[test]
    fn vision_fixture_validates_image_and_redacts_request_evidence() {
        let mut fixture = AiFixtureServer::start().unwrap();
        let image = format!("data:image/png;base64,{}", "a".repeat(256));
        let request = json!({"model":"fixture", "stream":false, "messages":[
            {"role":"system", "content":"Read the Chinese characters or words in the image"},
            {"role":"user", "content":[{"type":"image_url", "image_url":{"url":image}}]}
        ]});
        let response = exchange(fixture.port, "POST", "/v1/chat/completions", Some(request));
        assert!(response.contains("200 OK"), "{response}");
        assert!(response.contains("candidates"), "{response}");
        let evidence = exchange(fixture.port, "GET", "/__e2e/requests", None);
        assert!(evidence.contains("hanzi-recognition"), "{evidence}");
        assert!(evidence.contains("image omitted from E2E artifacts"));
        assert!(!evidence.contains("data:image/"));
        fixture.stop().unwrap();
    }

}
