use std::path::Path;

use zipeek_tts_stt::audio_codec::{InputFormat, decode_for_stt, wav_to_mp3, wav_to_ogg_opus};
use zipeek_tts_stt::candle_tts::qwen::{TtsProgress, synthesize_mandarin_with_progress};

#[test]
#[ignore = "requires downloaded Qwen3-TTS CustomVoice model and may take significant CPU/GPU time"]
fn qwen3_tts_generates_actual_non_silent_mandarin_wav() {
    let model = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let model_dir = model.parent().expect("Qwen3-TTS model parent");
    let device = std::env::var("ZIPEEK_TTS_DEVICE").unwrap_or_else(|_| "cuda".into());
    let mut events = Vec::new();
    let wav = synthesize_mandarin_with_progress(model_dir, "你好世界", &device, |event| {
        if let TtsProgress::GeneratingAcousticCodes { frames, .. } = &event {
            if *frames == 1 || *frames % 25 == 0 {
                eprintln!("Qwen3-TTS acoustic frames generated: {frames}");
            }
        } else {
            eprintln!("Qwen3-TTS progress: {event:?}");
        }
        events.push(event);
    })
    .expect("execute Qwen3-TTS talker, code predictor and neural audio decoder");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, TtsProgress::GeneratingAcousticCodes { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, TtsProgress::DecodingWaveform { .. }))
    );
    assert!(
        matches!(events.last(), Some(TtsProgress::Finished { wav_bytes }) if *wav_bytes == wav.len())
    );
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24_000);
    assert!(
        wav[44..]
            .chunks_exact(2)
            .any(|sample| i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs() > 200)
    );
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/speech-e2e");
    std::fs::create_dir_all(&output_dir).unwrap();

    let wav_path = output_dir.join("mandarin_nihao_shijie.wav");
    std::fs::write(&wav_path, &wav).unwrap();
    eprintln!("Qwen3-TTS neural WAV: {}", wav_path.display());

    let opus = wav_to_ogg_opus(&wav).expect("encode generated Qwen3-TTS WAV to Ogg Opus");
    assert!(opus.starts_with(b"OggS"));
    let decoded =
        decode_for_stt(&opus, InputFormat::OggOpus).expect("decode generated Qwen3-TTS Ogg Opus");
    assert!(!decoded.is_empty());
    assert!(decoded.iter().all(|sample| sample.is_finite()));
    assert!(decoded.iter().any(|sample| sample.abs() > 0.001));

    let opus_path = output_dir.join("mandarin_nihao_shijie.opus");
    std::fs::write(&opus_path, &opus).unwrap();
    eprintln!("Qwen3-TTS neural Ogg Opus: {}", opus_path.display());

    let mp3 = wav_to_mp3(&wav).expect("encode generated Qwen3-TTS WAV to MP3");
    let mp3_path = output_dir.join("mandarin_nihao_shijie.mp3");
    std::fs::write(&mp3_path, &mp3).unwrap();
    eprintln!("Qwen3-TTS neural MP3: {}", mp3_path.display());
    let decoded_mp3 =
        decode_for_stt(&mp3, InputFormat::Mp3).expect("decode generated Qwen3-TTS MP3");
    assert!(!decoded_mp3.is_empty());
    let expected_16khz_samples = ((wav.len() - 44) / 2) * 2 / 3;
    assert!(decoded_mp3.len().abs_diff(expected_16khz_samples) <= 2_400);
    assert!(decoded_mp3.iter().all(|sample| sample.is_finite()));
    assert!(decoded_mp3.iter().any(|sample| sample.abs() > 0.001));
}

#[test]
#[ignore = "requires downloaded Qwen3-TTS CustomVoice checkpoint and CUDA inference"]
fn qwen3_tts_synthesizes_mandarin_with_punctuation() {
    let model = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let model_dir = model.parent().expect("Qwen3-TTS model parent");
    let device = std::env::var("ZIPEEK_TTS_DEVICE").unwrap_or_else(|_| "cuda".into());
    let wav = zipeek_tts_stt::candle_tts::qwen::synthesize_mandarin(
        model_dir,
        "你好，欢迎学习中文。",
        &device,
    )
    .expect("synthesize Settings test sentence with Chinese punctuation");
    assert!(wav.len() > 44);
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert!(
        wav[44..]
            .chunks_exact(2)
            .any(|sample| i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs() > 200)
    );
}

#[test]
#[ignore = "requires downloaded 0.6B CustomVoice checkpoint and substantial memory; probes owned Candle talker and code predictor"]
fn real_checkpoint_maps_owned_talker_and_predictor() {
    let path = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let root = path.parent().expect("Qwen3-TTS model directory");
    let device = candle_core::Device::Cpu;
    let mut talker = zipeek_tts_stt::candle_tts::qwen::model::QwenTtsTalker::load(root, &device)
        .expect("load native Candle talker weights");
    let mut predictor =
        zipeek_tts_stt::candle_tts::qwen::model::QwenTtsCodePredictor::load(root, &device)
            .expect("load native Candle code predictor weights");
    let ids = talker.config.mandarin_vivian_codec_prefix();
    let inputs = talker
        .embed_codec(&ids, &device)
        .expect("codec conditioning embeddings");
    let (hidden, logits) = talker
        .forward_codec_embeddings(&inputs)
        .expect("run talker transformer");
    assert_eq!(logits.dims(), &[1, talker.config.talker.vocab_size]);
    let first_code = 0u32;
    let first_embed = talker.embed_codec(&[first_code], &device).unwrap();
    let talker_hidden = hidden.narrow(0, hidden.dim(0).unwrap() - 1, 1).unwrap();
    let codes = predictor
        .generate_greedy(first_code, &talker_hidden, &first_embed)
        .expect("run five-layer residual code predictor");
    assert_eq!(codes.len(), 16);
    talker.reset();
}

#[test]
#[ignore = "requires downloaded Qwen3-TTS speech tokenizer and substantial memory; executes native 16-codebook decoder"]
fn real_checkpoint_decodes_acoustic_codes_into_wav_samples() {
    let path = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let root = path.parent().expect("Qwen3-TTS model directory");
    let device = candle_core::Device::Cpu;
    let decoder = zipeek_tts_stt::candle_tts::qwen::codec::SpeechCodebookDecoder::load(
        root, &device,
    )
    .expect("load native Qwen3-TTS 12 Hz residual quantizer, codec Transformer and vocoder");
    let codes = vec![vec![0u32; 16]; 2];
    let wav = decoder
        .decode_waveform(&codes, &device)
        .expect("reconstruct waveform from 16 acoustic codebooks");
    assert_eq!(wav.len(), 2 * 1920);
    assert!(
        wav.iter()
            .all(|sample| sample.is_finite() && sample.abs() <= 1.0)
    );
}

#[test]
#[ignore = "requires the full Qwen3-TTS CustomVoice checkpoint and CUDA; checks engine reuse"]
fn qwen3_tts_engine_reuses_weights_and_resets_attention() {
    let checkpoint = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let device = std::env::var("ZIPEEK_TTS_DEVICE").unwrap_or_else(|_| "cuda".into());
    let mut engine = zipeek_tts_stt::candle_tts::qwen::QwenTtsEngine::load(
        checkpoint.parent().expect("model directory"),
        &device,
    )
    .expect("load persistent neural TTS engine");
    let first_start = std::time::Instant::now();
    let first = engine
        .synthesize("你好世界")
        .expect("first neural synthesis");
    let first_elapsed = first_start.elapsed();
    let warm_start = std::time::Instant::now();
    let second = engine
        .synthesize("你好世界")
        .expect("second neural synthesis");
    let warm_elapsed = warm_start.elapsed();
    let audio_duration_seconds = (second.len() - 44) as f64 / (2.0 * 24_000.0);
    eprintln!(
        "persistent Qwen3-TTS: first inference {:.3}s, warm inference {:.3}s, warm RTF {:.3}, audio {:.3}s",
        first_elapsed.as_secs_f64(),
        warm_elapsed.as_secs_f64(),
        warm_elapsed.as_secs_f64() / audio_duration_seconds,
        audio_duration_seconds,
    );
    assert_eq!(&first[..4], b"RIFF");
    assert_eq!(&second[..4], b"RIFF");
    assert_eq!(&first[8..12], b"WAVE");
    assert_eq!(&second[8..12], b"WAVE");
    assert!(first.len() > 44);
    assert!(second.len() > 44);
    assert!(first[44..].chunks_exact(2).any(|sample| i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs() > 200));
    assert!(second[44..].chunks_exact(2).any(|sample| i16::from_le_bytes([sample[0], sample[1]]).unsigned_abs() > 200));
}

#[tokio::test]
#[ignore = "requires downloaded Qwen3-TTS checkpoint and CUDA; verifies server startup warm-up"]
async fn speech_server_warms_tts_before_accepting_requests() {
    use zipeek_tts_stt::{server::SpeechServer, speech::Speech};

    let speech = Speech::new().expect("resolve speech model paths");
    assert!(speech.qwen_tts_checkpoint_present());
    assert!(!speech.neural_tts_engine_loaded());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback speech test listener");
    let server = SpeechServer::start(listener, speech.clone())
        .await
        .expect("load and warm up the neural engine before serving");
    assert!(speech.neural_tts_engine_loaded());
    if speech.candle_stt_checkpoint_present() {
        assert!(speech.neural_stt_engine_loaded());
    }
    let wav = speech.synthesize("你好世界").await.expect("post-warm-up neural synthesis");
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    server.shutdown().await.expect("stop warmed speech server");
}

#[test]
#[ignore = "requires Qwen3-TTS CustomVoice model and CUDA; diagnoses missing EOS for 不便"]
fn qwen3_tts_bubian_eos_diagnostic() {
    use zipeek_tts_stt::candle_tts::qwen::{
        model::{AcousticFrameDiagnostic, QwenTtsCodePredictor, QwenTtsTalker},
        tokenizer::MandarinBpeTokenizer,
    };

    let checkpoint = std::env::var_os("ZIPEEK_QWEN_TTS_MODEL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../models/speech/Qwen3-TTS-12Hz-0.6B-CustomVoice/model.safetensors")
        });
    let root = checkpoint.parent().expect("model directory");
    let device_name = std::env::var("ZIPEEK_TTS_DEVICE").unwrap_or_else(|_| "cuda".into());
    let device = match device_name.as_str() {
        "cuda" => candle_core::Device::new_cuda(0).expect("initialize CUDA device 0"),
        "cpu" => candle_core::Device::Cpu,
        other => panic!("unsupported ZIPEEK_TTS_DEVICE: {other}"),
    };
    let tokenizer =
        MandarinBpeTokenizer::load(root).expect("load Qwen3-TTS vocabulary and merges");
    let prompt = tokenizer
        .encode_assistant_mandarin("不便")
        .expect("tokenize regression input 不便");
    eprintln!(
        "Qwen3-TTS EOS diagnostic: text=不便, device={device_name}, prompt_token_ids={prompt:?}"
    );
    let mut talker = QwenTtsTalker::load(root, &device).expect("load Qwen3-TTS talker");
    let mut predictor =
        QwenTtsCodePredictor::load(root, &device).expect("load Qwen3-TTS code predictor");
    let max_frames = 375;
    let mut trace = Vec::<AcousticFrameDiagnostic>::new();
    let mut generated_frames = 0usize;
    let result = talker.generate_custom_voice_codes_with_diagnostics(
        &mut predictor,
        &prompt,
        &device,
        max_frames,
        &mut |count| generated_frames = count,
        &mut |frame| {
            if frame.frame <= 3 || frame.frame % 25 == 0 || frame.selected_eos {
                eprintln!(
                    "frame={:>3} argmax={} sampled={:?} best={:.4} eos={:.4} eos_minus_best={:.4} eos_selected={}",
                    frame.frame,
                    frame.winning_code,
                    frame.selected_code,
                    frame.winning_score,
                    frame.eos_score,
                    frame.eos_score - frame.winning_score,
                    frame.selected_eos,
                );
            }
            trace.push(frame);
        },
    );
    let longest_run = trace
        .chunk_by(|a, b| a.selected_code.is_some() && a.selected_code == b.selected_code)
        .map(|run| run.len())
        .max()
        .unwrap_or(0);
    let distinct_codes = trace
        .iter()
        .filter(|frame| !frame.selected_eos)
        .filter_map(|frame| frame.selected_code)
        .collect::<std::collections::HashSet<_>>()
        .len();
    let last = trace.last();
    eprintln!(
        "EOS diagnostic summary: observed_steps={}, generated_frames={}, distinct_first_codes={}, longest_repetition={}, last_frame={last:?}",
        trace.len(), generated_frames, distinct_codes, longest_run,
    );
    let codes = result.unwrap_or_else(|error| {
        panic!(
            "不便 failed acoustic generation: {error}; generated_frames={generated_frames}; observed_steps={}; distinct_codes={distinct_codes}; longest_repetition={longest_run}; last_frame={last:?}",
            trace.len(),
        )
    });
    assert!(
        !codes.is_empty(),
        "不便 must generate acoustic frames before EOS"
    );
    assert!(
        trace.last().is_some_and(|frame| frame.selected_eos),
        "不便 must terminate with EOS rather than reaching the frame limit"
    );
    assert!(codes.iter().all(|frame| frame.len() == 16));
}
