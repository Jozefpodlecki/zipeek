use std::{env, path::PathBuf, process};

use zipeek_tts_stt::candle_streaming_stt::{
    checkpoint::{StreamingCheckpointPaths, inspect_checkpoint},
    onnx_inspect::{Dimension, OnnxModelReport, export_embedded_initializers},
};

fn describe_dimension(dimension: &Dimension) -> String {
    match dimension {
        Dimension::Fixed(size) => size.to_string(),
        Dimension::Symbol(symbol) => symbol.clone(),
        Dimension::Unknown => "?".into(),
    }
}

fn print_model(name: &str, model: &OnnxModelReport) {
    println!(
        "{name}: IR v{}, {} nodes, {} initializers",
        model.ir_version,
        model.graph.node_count,
        model.graph.initializers.len()
    );
    for (direction, ports) in [
        ("input", &model.graph.inputs),
        ("output", &model.graph.outputs),
    ] {
        for port in ports {
            let dimensions = port
                .dimensions
                .iter()
                .map(describe_dimension)
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "  {direction} {} [{dimensions}] dtype={:?}",
                port.name, port.element_type
            );
        }
    }
    for (operator, count) in &model.graph.operators {
        println!("  operator {operator}: {count}");
    }
    for tensor in &model.graph.initializers {
        println!(
            "  weight {} {:?} dtype={} external={}",
            tensor.name, tensor.dimensions, tensor.data_type, tensor.external
        );
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() == 3 && arguments[0] == "--export" {
        let export = export_embedded_initializers(
            &PathBuf::from(&arguments[1]),
            &PathBuf::from(&arguments[2]),
        )?;
        println!(
            "Exported {} tensors ({} bytes) to {}",
            export.tensor_count, export.total_bytes, arguments[2]
        );
        return Ok(());
    }
    if arguments.len() != 4 && arguments.len() != 5 {
        return Err("usage: streaming_checkpoint_inspect ENCODER.onnx DECODER.onnx JOINER.onnx tokens.txt [blank_id] OR --export MODEL.onnx OUTPUT.safetensors".into());
    }
    let blank_id = match arguments.get(4) {
        Some(value) => value.parse::<u32>().map_err(|_| "invalid blank ID")?,
        None => 0,
    };
    let paths = StreamingCheckpointPaths {
        encoder: PathBuf::from(&arguments[0]),
        decoder: PathBuf::from(&arguments[1]),
        joiner: PathBuf::from(&arguments[2]),
        tokens: PathBuf::from(&arguments[3]),
        blank_id,
    };
    let report = inspect_checkpoint(&paths)?;
    println!(
        "Streaming model inventory: vocabulary={} blank_id={}",
        report.vocabulary_size, report.blank_id
    );
    print_model("encoder", &report.encoder);
    print_model("decoder", &report.decoder);
    print_model("joiner", &report.joiner);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
