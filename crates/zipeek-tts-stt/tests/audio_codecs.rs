mod common;

use common::{Server, voiced_wav};
use zipeek_tts_stt::audio_codec::{InputFormat, decode_for_stt, wav_to_mp3, wav_to_ogg_opus};

const REFERENCE_MP3: &[u8] = include_bytes!("fixtures/mandarin_codec_input.mp3");
const REFERENCE_FLAC: &[u8] = include_bytes!("fixtures/mandarin_codec_input.flac");

#[test]
fn mp3_and_flac_decode_into_bounded_mono_16khz_pcm() {
    for (format, bytes) in [
        (InputFormat::Mp3, REFERENCE_MP3),
        (InputFormat::Flac, REFERENCE_FLAC),
    ] {
        let pcm = decode_for_stt(bytes, format).expect("decode fixture without native libraries");
        assert!((18_000..=23_000).contains(&pcm.len()));
        assert!(pcm.iter().all(|sample| sample.is_finite()));
        assert!(pcm.iter().any(|sample| sample.abs() > 0.005));
    }
    assert!(decode_for_stt(REFERENCE_FLAC, InputFormat::Mp3).is_err());
    assert!(decode_for_stt(REFERENCE_MP3, InputFormat::Flac).is_err());
}

#[test]
fn opus_export_can_be_read_back_as_stt_audio() {
    let wave = voiced_wav();
    let ogg = wav_to_ogg_opus(&wave).expect("encode fixed PCM to Ogg Opus");
    assert!(ogg.starts_with(b"OggS"));
    let pcm = decode_for_stt(&ogg, InputFormat::OggOpus).expect("decode Ogg Opus to STT PCM");
    assert_eq!(pcm.len(), 8_000);
    assert!(pcm.iter().any(|sample| sample.abs() > 0.005));
}

#[test]
fn mp3_export_can_be_read_back_as_stt_audio() {
    let wave = voiced_wav();
    let mp3 = wav_to_mp3(&wave).expect("encode fixed PCM to MP3");
    assert!(!mp3.is_empty());
    let decoded = decode_for_stt(&mp3, InputFormat::Mp3).expect("decode MP3 to STT PCM");
    assert!(!decoded.is_empty());
    assert!(decoded.iter().all(|sample| sample.is_finite()));
    assert!(decoded.iter().any(|sample| sample.abs() > 0.005));
}

#[test]
fn long_mp3_export_decodes_without_truncation() {
    let wav = voiced_wav();
    let data = &wav[44..];
    let mut extended = wav[..44].to_vec();
    for _ in 0..4 {
        extended.extend_from_slice(data);
    }
    let data_bytes = (extended.len() - 44) as u32;
    extended[4..8].copy_from_slice(&(36 + data_bytes).to_le_bytes());
    extended[40..44].copy_from_slice(&data_bytes.to_le_bytes());
    let mp3 = wav_to_mp3(&extended).expect("encode longer PCM to MP3");
    let decoded = decode_for_stt(&mp3, InputFormat::Mp3).expect("decode longer MP3");
    assert!(decoded.len() > 16_000);
    assert!(decoded.iter().all(|sample| sample.is_finite()));
    assert!(decoded.iter().any(|sample| sample.abs() > 0.005));
}

#[test]
fn mp3_encoder_handles_neural_tts_length_and_rejects_bad_input() {
    let wav = voiced_wav();
    let base = &wav[44..];
    let samples_to_emit = 42_240usize;
    let mut generated = wav[..44].to_vec();
    for index in 0..samples_to_emit {
        generated.extend_from_slice(&base[(index % (base.len() / 2)) * 2..][..2]);
    }
    let length = (samples_to_emit * 2) as u32;
    generated[4..8].copy_from_slice(&(36 + length).to_le_bytes());
    generated[40..44].copy_from_slice(&length.to_le_bytes());
    let mp3 = wav_to_mp3(&generated).expect("2.64-second synthetic speech encodes to valid MP3");
    let decoded = decode_for_stt(&mp3, InputFormat::Mp3)
        .expect("2.64-second MP3 decodes through the speech decoder");
    assert!(decoded.len().abs_diff(samples_to_emit) <= 3_200);
    assert!(decoded.iter().all(|sample| sample.is_finite()));
    assert!(decoded.iter().any(|sample| sample.abs() > 0.005));
    assert!(wav_to_mp3(b"not a wav").is_err());
}

#[test]
fn http_accepts_encoded_audio_without_reporting_transcription_success() {
    let server = Server::start_without_stt_model();
    for (mime, bytes) in [
        ("audio/mpeg", REFERENCE_MP3),
        ("audio/flac", REFERENCE_FLAC),
    ] {
        let response = server.request("POST", "/v1/stt", &[("Content-Type", mime)], bytes);
        assert_eq!(
            response.status, 503,
            "missing checkpoint should not report successful STT"
        );
    }
    let response = server.request(
        "POST",
        "/v1/stt",
        &[("Content-Type", "audio/ogg")],
        b"invalid ogg bytes",
    );
    assert_eq!(response.status, 400);
    let response = server.request(
        "POST",
        "/v1/stt",
        &[("Content-Type", "application/octet-stream")],
        REFERENCE_MP3,
    );
    assert_eq!(response.status, 400);
}

#[test]
fn tts_supports_explicit_ogg_opus_content_negotiation() {
    let server = Server::start_without_stt_model();
    let unsupported = server.request(
        "POST",
        "/v1/tts",
        &[
            ("Content-Type", "text/plain; charset=utf-8"),
            ("Accept", "audio/aac"),
        ],
        "你好".as_bytes(),
    );
    assert_eq!(unsupported.status, 406);
    let capabilities = server.request("GET", "/v1/capabilities", &[], &[]).text();
    let response = server.request(
        "POST",
        "/v1/tts",
        &[
            ("Content-Type", "text/plain; charset=utf-8"),
            ("Accept", "audio/ogg"),
        ],
        "你好".as_bytes(),
    );
    if capabilities.contains("synthesis=true\n") {
        assert_eq!(response.status, 200, "{}", response.text());
        assert_eq!(response.header("content-type"), Some("audio/ogg"));
        assert!(response.body.starts_with(b"OggS"));
        let decoded = decode_for_stt(&response.body, InputFormat::OggOpus).unwrap();
        assert!(decoded.iter().any(|sample| sample.abs() > 0.005));
    } else {
        assert_eq!(response.status, 503);
    }
}
