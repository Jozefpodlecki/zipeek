import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("encoder", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    import numpy as np
    import onnxruntime as ort

    if arguments.output.exists():
        parser.error("output already exists; refusing to overwrite a reference fixture")
    session = ort.InferenceSession(str(arguments.encoder), providers=["CPUExecutionProvider"])
    rng = np.random.default_rng(42)
    inputs = {}
    for port in session.get_inputs():
        dimensions = [1 if isinstance(value, str) or value is None else value for value in port.shape]
        if port.name == "x":
            if dimensions != [1, 77, 80]:
                parser.error(f"unexpected Zipformer feature shape: {dimensions}")
            inputs[port.name] = rng.normal(0.0, 0.3, dimensions).astype(np.float32)
        elif port.name == "processed_lens":
            inputs[port.name] = np.zeros(dimensions, dtype=np.int64)
        else:
            inputs[port.name] = np.zeros(dimensions, dtype=np.float32)
    expected_shape = (1, 128, 3, 19)
    if inputs["embed_states"].shape != expected_shape:
        parser.error("checkpoint embedding cache has unexpected shape")
    cache = session.run(["new_embed_states"], inputs)[0]
    if cache.shape != expected_shape or not np.isfinite(cache).all():
        parser.error("unexpected or non-finite embedding cache result")
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    with arguments.output.open("x", encoding="utf-8") as stream:
        json.dump({
            "features": inputs["x"].ravel().astype(float).tolist(),
            "next_embed_states": cache.ravel().astype(float).tolist(),
        }, stream, allow_nan=False, separators=(",", ":"))
    print(f"Wrote streaming embedding reference: features {inputs['x'].shape}, cache {cache.shape}")


if __name__ == "__main__":
    main()
