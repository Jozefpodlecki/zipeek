# Native streaming ASR checkpoint preflight (R680)

This document specifies the inspection boundary for the provisional `pkufool/zipformer-small-streaming` model. The published repository lists separate `encoder-chunk-16-left-64.onnx`, `decoder-chunk-16-left-64.onnx`, `joiner-chunk-16-left-64.onnx` files and `data/tokens.txt`. The upstream page currently lists an Apache-2.0 license, but the full revision SHA, each downloaded file hash, and any redistributed assets must be verified before packaging. The model repository contains TorchScript artifacts flagged as suspicious by the hosting service. **Do not deserialize or execute untrusted `.pt`/pickle files for this workflow.** The first conversion path uses data-only ONNX initializers.

Reference:
- https://huggingface.co/pkufool/zipformer-small-streaming/tree/main
- https://huggingface.co/pkufool/zipformer-small-streaming/tree/main/data

## Windows local acquisition and inventory

Download the chosen ONNX exports and `data/tokens.txt` from a pinned repository revision using a trusted Hugging Face download workflow. Do not place them under packaged web assets or commit them to the source archive. Preserve the exact source revision and record all SHA-256 hashes (for example, `Get-FileHash path -Algorithm SHA256`). The directory names and archive layout are not a checkpoint loader contract.

Run the new source-local inspection program:

```powershell
cargo run --locked -p zipeek-tts-stt --bin streaming_checkpoint_inspect -- `
  "C:\models\zipformer\encoder-chunk-16-left-64.onnx" `
  "C:\models\zipformer\decoder-chunk-16-left-64.onnx" `
  "C:\models\zipformer\joiner-chunk-16-left-64.onnx" `
  "C:\models\zipformer\data\tokens.txt" 0 `
  *> target\zipformer-r680-inventory.txt
```

The optional last argument is the blank token ID; 0 is a diagnostic starting value, **not** a verified model fact. Confirm the upstream tokenizer's blank convention. The inspector reads protobuf metadata without invoking an ONNX execution engine or deserializing PyTorch objects. It reports graph input/output shapes (including symbolic cache dimensions), operator counts, initialized tensor names, shapes, and types. It rejects missing or malformed graphs, external initializers, incompatible ONNX IR revisions across the three components and an inconsistent fixed joiner-vocabulary dimension. The inspector does not prove an executable Candle graph.

For each ONNX component, a data-only conversion can be attempted:

```powershell
cargo run --locked -p zipeek-tts-stt --bin streaming_checkpoint_inspect -- `
  --export "C:\models\zipformer\encoder-chunk-16-left-64.onnx" `
  "C:\models\zipformer\encoder.safetensors"
```

Repeat for decoder and joiner with distinct output paths. The exporter copies supported, uncompressed, embedded `TensorProto.raw_data` bytes into the safetensors format, validates every shape/byte count, and refuses an existing output filename. It refuses external data, unsupported dtypes and tensors stored only as typed protobuf fields. This converter does **not** convert graph operations, fold constants, implement Zipformer layers or validate numeric parity. Large or nonstandard ONNX exports may require a separately approved conversion procedure. Never pass an ONNX graph directly to Candle's `VarBuilder`.

## Engineering milestones still blocked by model artifacts

1. Run the inventory against actual files and retain the report, exact SHA-256 hashes, model revision and applicable notices.
2. Determine which graph initializers represent trained weights versus constants, and map their exact namespaces to Candle modules. Use initializer inventory and graph operators rather than guessing an architecture from filenames.
3. Implement the selected checkpoint's convolutional subsampler, per-stack Zipformer encoder layers and streaming-cache tensors with exact cache sizes and offsets.
4. Establish frontend parity against the model's online Kaldi/Fbank implementation, especially `snip_edges`, framing, scale, dither and final padding.
5. Use the same PCM samples to compare reference ONNX and Candle intermediate tensors and token emissions. Quantify absolute/relative error and check multiple chunk partitions.
6. Only after parity and Mandarin decoding tests pass, connect persistent streaming model sessions to bounded transport and the experimental Live tab.

R680 supplies model-file inspection and conversion primitives only. It does **not** claim a working Zipformer recognizer, an implemented checkpoint-compatible encoder, or runtime equivalence with ONNX.
