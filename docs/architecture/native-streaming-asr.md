# Native streaming ASR: R682 checkpoint-specific head and cache contract

R682 uses the **user-supplied Windows ONNX inventory** from the cloned `pkufool/zipformer-small-streaming` `chunk-32-left-128` files. This is checkpoint inspection evidence, not a model execution or verified pinned upstream commit. The existing Qwen3-ASR and TTS production paths are untouched.

The encoder declares 77 input frames with 80 mel bins per invocation, a 512-wide output, twelve attention/cache groups, embedding state `[N,128,3,19]` and processed lengths `int64[N]`. The new `zipformer32.rs` module describes and validates 12 groups of keys, nonlinear attention, values and convolution histories. The group sizes are **not uniform**. Initial cache states allocate zero tensors on the selected Candle device. The model-specific encoder operations, graph output-to-cache transfers and exact frontend remain outstanding.

The checkpoint's decoder exports `[N,2]` int64 contexts, `[N,512]` output, embedding `[11661,512]`, grouped convolution `[512,4,2]`, and projection `[512,512]`. The joiner exports `Add -> Tanh -> Gemm` and only the `[11661,512]` output projection, not the three projections assumed by the earlier generic prototype. `checkpoint_head.rs` is a separate checkpoint-shaped Candle implementation; it has not established numeric parity with ONNX, including exact ReLU placement and token preprocessing. The generic prototype is retained separately for its synthetic-pipeline tests, not represented as compatible.

Weight conversion to safetensors requires the model files in the local `models/speech` checkout. Neither weights nor the 1.77 GiB Hugging Face clone are bundled in releases. Before enabling recognition, record the cloned commit hash, verify frontend parity, run ONNX vs Candle decoder/joiner logits on fixed tokens and encoder features, implement the complete Zipformer2 encoder/cache updates and test streaming Mandarin transcription. Do not add a WebSocket service or Live tab until those pass.

## Previous R680 checkpoint integration preparation

The `candle_streaming_stt` subsystem remains a non-production experimental module. It does not replace native Qwen3-ASR, and it has no HTTP/WebSocket endpoint or UI route. R679 provided incremental 16 kHz PCM framing, provisional Fbank extraction, a token-vocabulary reader and a streaming pipeline with a synthetic encoder. R680 adds a strict ONNX checkpoint inspector and a data-only embedded-initializer exporter so the actual encoder, decoder, joiner and cache interfaces can be mapped before implementing checkpoint-specific Candle math.

The provisional model is `pkufool/zipformer-small-streaming`. The upstream repository lists Chinese/English streaming models under Apache-2.0 and separate transducer encoder/decoder/joiner ONNX components for chunk 16/left 64, with a `data/tokens.txt` file. The upstream `model.pt` / TorchScript files are not parsed or executed. No model weights have been acquired or bundled in R680. Actual immutable revision, asset checksums and licensing provenance remain to be recorded locally.

See [checkpoint preflight](native-streaming-checkpoint-preflight.md) for local inspection/export commands, strict file-format limitations and the remaining acceptance path.

## Supported implementation

- R678: bounded PCM queue, transducer decoding state, native Candle predictor/joiner prototypes and generic attention-cache primitive.
- R679: incremental provisional Fbank, strict dense token-map parser, and test-injected encoder pipeline.
- R680: dependency-free, bounded protobuf inspection of ONNX metadata (graph ports, symbolic dimensions, operator inventory and initializer shapes/dtypes), component-set preflight, and byte-checked export of supported embedded initializers to safetensors.

## Critical limitations

- No Zipformer2 checkpoint-backed encoder is implemented. The generic attention cache is not a Zipformer layer.
- The decoder/joiner namespaces remain provisional and are not mapped to verified upstream initializer names.
- Exporting weight bytes is not equivalent to converting an ONNX computation graph into Candle operators.
- R679 feature extraction is not verified to match the upstream online Fbank.
- An ONNX graph inventory is an engineering input, not numerical parity evidence.
- Rust compilation, unit tests, checkpoint conversion and CUDA validation have not run in this authoring environment.

The next release should consume a pinned checkpoint inventory and target exact encoder modules and cache conventions. Do not add WebSocket transport or Live UI before genuine incremental model inference works.

## R683: checkpoint-backed decoder and joiner parity gate

R683 corrects the checkpoint decoder convolution to `conv1d_no_bias`. The exported decoder contains `decoder.conv.weight` `[512,4,2]` and **no** `decoder.conv.bias`, so a bias-requiring Candle constructor cannot load the real checkpoint. The existing grouping (128) and kernel (2) are preserved.

The new `streaming_head_parity` command maps the converted safetensors through Candle, evaluates three fixed token contexts, and compares all 512 decoder outputs and all 11,661 joiner logits against an ONNX Runtime-generated JSON fixture. The reference-generation script is optional developer tooling and uses a separately installed Python `numpy` and `onnxruntime`; neither is added to the Rust dependency graph or production application.

Run from the repository root in PowerShell, after installing the optional Python packages in a dedicated local environment:

```powershell
$checkpoint = 'models/speech/zipformer-small-streaming'
New-Item -ItemType Directory -Force target/zipformer-parity | Out-Null
cargo run --locked -p zipeek-tts-stt --bin streaming_checkpoint_inspect -- --export "$checkpoint/decoder-chunk-32-left-128.onnx" target/zipformer-parity/decoder.safetensors
cargo run --locked -p zipeek-tts-stt --bin streaming_checkpoint_inspect -- --export "$checkpoint/joiner-chunk-32-left-128.onnx" target/zipformer-parity/joiner.safetensors
python scripts/zipformer-head-reference.py "$checkpoint/decoder-chunk-32-left-128.onnx" "$checkpoint/joiner-chunk-32-left-128.onnx" target/zipformer-parity/reference.json
cargo run --locked -p zipeek-tts-stt --bin streaming_head_parity -- target/zipformer-parity/decoder.safetensors target/zipformer-parity/joiner.safetensors target/zipformer-parity/reference.json
```

The export command refuses to overwrite existing output, so remove or rename only the **generated** target/zipformer-parity files before repeating. The parity command fails on missing tensors, malformed or nonfinite reference values, or mismatched decoder vectors or joiner logits. Its tolerances (decoder `2e-4 + 2e-4 * abs(reference)`; joiner `2e-3 + 5e-4 * abs(reference)`) are initial CPU tolerances, not a claim of verified numeric equivalence. Explicitly capture the console output and checkpoint SHA-256 hashes. The experimental pipeline cannot transcribe until the encoder and streaming-state transfer are fully implemented.


## R686 experimental WebSocket transport

`GET /v1/stt/live` is an experimental localhost-only WebSocket route backed by the persistent Qwen3-ASR worker for best-effort partial/final transcripts. It does **not** use the unfinished Zipformer encoder. See root `RELEASE_NOTES.md` for the frame protocol, limits and initial acceptance commands. This represents a deliberate, separately approved transport milestone before full native model parity; the R682 historical warning against enabling an unverified Zipformer model remains valid. The R685 embedding source and parity tools have been restored into the Tauri mainline, but remain unverified on Windows.
