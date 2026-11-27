use std::path::{Path, PathBuf};

use super::{
    onnx_inspect::{Dimension, OnnxModelReport, inspect_file},
    tokens::TransducerTokens,
};

#[derive(Clone, Debug)]
pub struct StreamingCheckpointPaths {
    pub encoder: PathBuf,
    pub decoder: PathBuf,
    pub joiner: PathBuf,
    pub tokens: PathBuf,
    pub blank_id: u32,
}

#[derive(Clone, Debug)]
pub struct StreamingCheckpointReport {
    pub encoder: OnnxModelReport,
    pub decoder: OnnxModelReport,
    pub joiner: OnnxModelReport,
    pub vocabulary_size: usize,
    pub blank_id: u32,
}

pub fn inspect_checkpoint(
    paths: &StreamingCheckpointPaths,
) -> Result<StreamingCheckpointReport, String> {
    let encoder = inspect_component(&paths.encoder, "encoder")?;
    let decoder = inspect_component(&paths.decoder, "decoder")?;
    let joiner = inspect_component(&paths.joiner, "joiner")?;
    let vocabulary = TransducerTokens::from_file(&paths.tokens, paths.blank_id)?;
    if encoder.graph.inputs.is_empty()
        || encoder.graph.outputs.is_empty()
        || decoder.graph.inputs.is_empty()
        || decoder.graph.outputs.is_empty()
        || joiner.graph.inputs.is_empty()
        || joiner.graph.outputs.is_empty()
    {
        return Err(
            "one or more streaming model components lack declared inputs or outputs".into(),
        );
    }
    if encoder.ir_version != decoder.ir_version || decoder.ir_version != joiner.ir_version {
        return Err("streaming model components have inconsistent ONNX IR versions".into());
    }
    if let Some(Dimension::Fixed(size)) = joiner
        .graph
        .outputs
        .first()
        .and_then(|port| port.dimensions.last())
    {
        if *size > 1 && *size != vocabulary.len() as u64 {
            return Err(format!(
                "joiner output vocabulary dimension {size} differs from tokens.txt size {}",
                vocabulary.len()
            ));
        }
    }
    Ok(StreamingCheckpointReport {
        encoder,
        decoder,
        joiner,
        vocabulary_size: vocabulary.len(),
        blank_id: paths.blank_id,
    })
}

fn inspect_component(path: &Path, name: &str) -> Result<OnnxModelReport, String> {
    let report = inspect_file(path).map_err(|error| format!("{name} ONNX inspection: {error}"))?;
    if report.graph.node_count == 0 {
        return Err(format!("{name} model has no graph operators"));
    }
    if report
        .graph
        .initializers
        .iter()
        .any(|tensor| tensor.external)
    {
        return Err(format!(
            "{name} model uses externally stored weights; bundle verification required"
        ));
    }
    Ok(report)
}
