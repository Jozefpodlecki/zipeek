"""Generate optional ONNX Runtime numerical references for Zipeek's Candle head parity CLI.

Requires numpy and onnxruntime in the invoking Python environment. Not a runtime dependency.
"""
import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("decoder", type=Path)
    parser.add_argument("joiner", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    import numpy as np
    import onnxruntime as ort

    decoder = ort.InferenceSession(str(args.decoder), providers=["CPUExecutionProvider"])
    joiner = ort.InferenceSession(str(args.joiner), providers=["CPUExecutionProvider"])
    if args.output.exists():
        parser.error("output already exists; refusing to overwrite a parity fixture")
    cases = []
    for seed, tokens in [(1, [0, 2]), (2, [11, 37]), (3, [100, 500])]:
        rng = np.random.default_rng(seed)
        encoded = rng.normal(0.0, 0.25, (1, 512)).astype(np.float32)
        context = np.array([tokens], dtype=np.int64)
        decoded = decoder.run(None, {decoder.get_inputs()[0].name: context})[0]
        logits = joiner.run(None, {
            joiner.get_inputs()[0].name: encoded,
            joiner.get_inputs()[1].name: decoded,
        })[0]
        assert decoded.shape == (1, 512), decoded.shape
        assert logits.shape == (1, 11661), logits.shape
        cases.append({
            "tokens": tokens,
            "encoder": encoded[0].astype(float).tolist(),
            "decoder": decoded[0].astype(float).tolist(),
            "logits": logits[0].astype(float).tolist(),
        })
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump({"cases": cases}, stream, allow_nan=False, separators=(",", ":"))
    print(f"Wrote {len(cases)} reference cases to {args.output}")


if __name__ == "__main__":
    main()
