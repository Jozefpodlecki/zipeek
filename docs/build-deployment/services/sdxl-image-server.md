# Local SDXL Base inference

The `zipeek-image-server` is a native, loopback-only Axum service.
The default backend remains the deterministic SVG placeholder. Select `sdxl`
explicitly to generate actual PNG images using Candle 0.11.1 and CUDA.

## Local model assets

The SDXL backend reads files **only** from disk, with no automatic downloads.
Default root: `<Cargo workspace>/models/sdxl` (resolved from the image-server crate manifest, independent of the launch directory). Set `ZIPEEK_IMAGE_MODEL_DIR` to override.

```text
sdxl/
  base/
    unet/diffusion_pytorch_model.fp16.safetensors
    text_encoder/model.fp16.safetensors
    text_encoder_2/model.fp16.safetensors
  vae/diffusion_pytorch_model.safetensors
  clip/tokenizer.json
  clip2/tokenizer.json
```

The VAE file is from `madebyollin/sdxl-vae-fp16-fix`; the tokenizers are
from `openai/clip-vit-large-patch14` and
`laion/CLIP-ViT-bigG-14-laion2B-39B-b160k`. These match Candle's SDXL
Stable Diffusion example. Download and review model licences separately.

Direct file sources (download only any files you do not have):
- SDXL FP16 base weights: https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/tree/main
  (`unet/diffusion_pytorch_model.fp16.safetensors`, `text_encoder/model.fp16.safetensors`,
  `text_encoder_2/model.fp16.safetensors`). The unqualified UNet file without `.fp16.`
  is **not** the expected FP16 file and is much larger.
- VAE: https://huggingface.co/madebyollin/sdxl-vae-fp16-fix/blob/main/diffusion_pytorch_model.safetensors
- CLIP 1 tokenizer: https://huggingface.co/openai/clip-vit-large-patch14/blob/main/tokenizer.json
- CLIP 2 tokenizer: https://huggingface.co/laion/CLIP-ViT-bigG-14-laion2B-39B-b160k/blob/main/tokenizer.json

The first two encoder weights already on disk can be reused; only download missing
files. The image server validates its six required files at startup and reports a
structured `SdxlSetupError` when any is missing, empty, or not a regular file.

**Prompt length:** The Rust backend accepts up to 300 content tokens
per tokenizer. Each CLIP encoder processes repeated windows of up to 75 content
tokens plus its two special tokens, preserving the model's 77-position context.
All encoded windows are concatenated along the sequence dimension and passed to
the U-Net, with the two SDXL encoders and unconditional guidance aligned to the
same window count. This avoids the old silent 77-token truncation and needs no
new weights. A prompt beyond 300 content tokens fails explicitly.

Long prompts are a conditioning technique rather than equivalent to a model
trained with a longer attention context. Semantics may weaken across chunks and
VRAM/latency increase with prompt length. The live CUDA smoke test remains
opt-in; no runtime inference or image-quality acceptance has been completed in
this packaging environment.

```powershell
cargo test --locked -p zipeek-image-server clip_chunks_preserve_all_250_content_tokens
cargo test --locked -p zipeek-image-server sdxl_generates_png_from_250_token_prompt -- --ignored --nocapture
```

## Run on Windows PowerShell

The PNG encoder uses `flate2` and `crc32fast`, both already present in
the uploaded workspace lockfile. updates the image-server lockfile entry
to include them and `tokenizers`. Validate the pinned dependency graph:

```powershell
cargo check -p zipeek-image-server
cargo test -p zipeek-image-server
cargo make image-server-sdxl
```

For a custom model directory or step count:

```powershell
$env:ZIPEEK_IMAGE_MODEL_DIR = "C:\zipeek-models\sdxl"
$env:ZIPEEK_IMAGE_STEPS = "20"
cargo make image-server-sdxl
```

Send a request from another terminal:

```powershell
$job = Invoke-RestMethod -Method Post `
  -Uri http://127.0.0.1:1431/v1/images `
  -ContentType application/json `
  -Body '{"prompt":"A red apple on a plain wooden table, product photograph, no text","width":512,"height":512}'
$job
do {
  Start-Sleep -Seconds 3
  $status = Invoke-RestMethod "http://127.0.0.1:1431/v1/jobs/$($job.id)"
  $status
} while ($status.status -in @("queued", "running"))
if ($status.status -eq "completed") {
  Invoke-WebRequest "http://127.0.0.1:1431$($status.image_url)" -OutFile "$PWD\sdxl-test.png"
}
```

`GET /health` reports `backend: sdxl` when selected.

## Limits and known caveats

- **Not CUDA verified in the release-build environment**: compile and generate
  on the Windows RTX 5080 laptop before considering inference operational.
- One concurrent generator; four total queued/running jobs. Inference is run on
  a Tokio blocking worker; it does not block the web executor.
- The initial implementation reconstructs Candle weights for each request,
  keeping the architecture simple at the cost of warm-up time. Model lifetime
  caching can be added after an actual CUDA memory profile.
- It uses the official Candle SDXL dual-CLIP, DDIM, UNet and VAE path,
  with FP16 UNet/VAE, CLIP inference in F32 and guidance 7.5.
- CUDA generation can exhaust VRAM, especially at 768px. Start with 512px,
  batch 1, and unload large language models first.
- Jobs are in memory across server restarts; completed images and metadata
  persist in `ZIPEEK_IMAGE_DATA_DIR` as `.png` or `.svg` plus `.json`.
- Prompts and images are local. No remote model/API calls occur.
- Image exercise answer correctness and visual validation are separate work.

## Source provenance

A full pre-reconciliation copy is retained in [historical source evidence](../../archive-history/audits/maintained-source-snapshots/build-deployment-services-sdxl-image-server.md). Dated implementation claims in that snapshot are not current build or browser verification.
