use std::{collections::BTreeMap, fs, path::Path};

const MAX_MODEL_BYTES: usize = 256 * 1024 * 1024;
const MAX_FIELDS: usize = 1_000_000;
const MAX_DEPTH: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dimension {
    Fixed(u64),
    Symbol(String),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorPort {
    pub name: String,
    pub element_type: Option<u64>,
    pub dimensions: Vec<Dimension>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TensorWeight {
    pub name: String,
    pub data_type: u64,
    pub dimensions: Vec<u64>,
    pub external: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OnnxGraphReport {
    pub name: String,
    pub inputs: Vec<TensorPort>,
    pub outputs: Vec<TensorPort>,
    pub initializers: Vec<TensorWeight>,
    pub operators: BTreeMap<String, usize>,
    pub node_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnnxModelReport {
    pub ir_version: u64,
    pub graph: OnnxGraphReport,
}

struct Wire<'a> {
    bytes: &'a [u8],
    offset: usize,
    fields: usize,
}

impl<'a> Wire<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            fields: 0,
        }
    }

    fn varint(&mut self) -> Result<u64, String> {
        let mut value = 0u64;
        for index in 0..10 {
            let byte = *self
                .bytes
                .get(self.offset)
                .ok_or("truncated protobuf varint")?;
            self.offset += 1;
            if index == 9 && byte > 1 {
                return Err("protobuf varint overflow".into());
            }
            value |= u64::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("protobuf varint overflow".into())
    }

    fn next(&mut self) -> Result<Option<(u32, Field<'a>)>, String> {
        if self.offset == self.bytes.len() {
            return Ok(None);
        }
        self.fields += 1;
        if self.fields > MAX_FIELDS {
            return Err("ONNX metadata exceeds field count limit".into());
        }
        let tag = self.varint()?;
        let number = u32::try_from(tag >> 3).map_err(|_| "invalid protobuf tag")?;
        if number == 0 {
            return Err("zero protobuf field number".into());
        }
        let value = match tag & 7 {
            0 => Field::Integer(self.varint()?),
            1 | 5 => {
                let size = if tag & 7 == 1 { 8 } else { 4 };
                let end = self
                    .offset
                    .checked_add(size)
                    .ok_or("protobuf offset overflow")?;
                if end > self.bytes.len() {
                    return Err("truncated fixed-width protobuf field".into());
                }
                self.offset = end;
                Field::Other
            }
            2 => {
                let length =
                    usize::try_from(self.varint()?).map_err(|_| "protobuf length overflow")?;
                let end = self
                    .offset
                    .checked_add(length)
                    .ok_or("protobuf offset overflow")?;
                let bytes = self
                    .bytes
                    .get(self.offset..end)
                    .ok_or("truncated protobuf field")?;
                self.offset = end;
                Field::Bytes(bytes)
            }
            _ => return Err("unsupported protobuf wire type".into()),
        };
        Ok(Some((number, value)))
    }
}

enum Field<'a> {
    Integer(u64),
    Bytes(&'a [u8]),
    Other,
}

fn utf8(value: &[u8]) -> Result<String, String> {
    std::str::from_utf8(value)
        .map(str::to_owned)
        .map_err(|_| "invalid ONNX UTF-8".into())
}

fn parse_message<T: Default>(
    bytes: &[u8],
    depth: usize,
    mut visit: impl FnMut(&mut T, u32, Field<'_>) -> Result<(), String>,
) -> Result<T, String> {
    if depth > MAX_DEPTH {
        return Err("ONNX metadata nesting limit exceeded".into());
    }
    let mut result = T::default();
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.next()? {
        visit(&mut result, number, value)?;
    }
    Ok(result)
}

fn tensor_dimension(bytes: &[u8]) -> Result<Dimension, String> {
    parse_message(bytes, 7, |dimension, number, value| {
        match (number, value) {
            (1, Field::Integer(n)) => *dimension = Dimension::Fixed(n),
            (2, Field::Bytes(name)) => *dimension = Dimension::Symbol(utf8(name)?),
            _ => {}
        }
        Ok(())
    })
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Unknown
    }
}

fn tensor_shape(bytes: &[u8]) -> Result<Vec<Dimension>, String> {
    parse_message(bytes, 6, |result: &mut Vec<Dimension>, number, field| {
        if let (1, Field::Bytes(bytes)) = (number, field) {
            result.push(tensor_dimension(bytes)?);
        }
        Ok(())
    })
}

fn tensor_type(bytes: &[u8], port: &mut TensorPort) -> Result<(), String> {
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.next()? {
        if let (1, Field::Bytes(tensor)) = (number, value) {
            let mut tensor_fields = Wire::new(tensor);
            while let Some((field_id, field_value)) = tensor_fields.next()? {
                match (field_id, field_value) {
                    (1, Field::Integer(dtype)) => port.element_type = Some(dtype),
                    (2, Field::Bytes(shape)) => port.dimensions = tensor_shape(shape)?,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn value_info(bytes: &[u8]) -> Result<TensorPort, String> {
    parse_message(bytes, 3, |port: &mut TensorPort, number, value| {
        match (number, value) {
            (1, Field::Bytes(name)) => port.name = utf8(name)?,
            (2, Field::Bytes(ty)) => tensor_type(ty, port)?,
            _ => {}
        }
        Ok(())
    })
}

impl Default for TensorPort {
    fn default() -> Self {
        Self {
            name: String::new(),
            element_type: None,
            dimensions: Vec::new(),
        }
    }
}

fn initializer(bytes: &[u8]) -> Result<TensorWeight, String> {
    let mut output = TensorWeight {
        name: String::new(),
        data_type: 0,
        dimensions: Vec::new(),
        external: false,
    };
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.next()? {
        match (number, value) {
            (1, Field::Integer(dimension)) => output.dimensions.push(dimension),
            (1, Field::Bytes(packed)) => {
                let mut values = Wire::new(packed);
                while values.offset < packed.len() {
                    output.dimensions.push(values.varint()?);
                }
            }
            (2, Field::Integer(dtype)) => output.data_type = dtype,
            (8, Field::Bytes(name)) => output.name = utf8(name)?,
            (14, Field::Integer(location)) => output.external = location == 1,
            _ => {}
        }
    }
    if output.name.is_empty() {
        return Err("ONNX initializer without name".into());
    }
    Ok(output)
}

fn node_op(bytes: &[u8]) -> Result<String, String> {
    let mut wire = Wire::new(bytes);
    while let Some((number, field)) = wire.next()? {
        if let (4, Field::Bytes(op)) = (number, field) {
            return utf8(op);
        }
    }
    Err("ONNX node without op_type".into())
}

fn graph(bytes: &[u8]) -> Result<OnnxGraphReport, String> {
    parse_message(bytes, 2, |result: &mut OnnxGraphReport, number, value| {
        match (number, value) {
            (1, Field::Bytes(node)) => {
                let op = node_op(node)?;
                *result.operators.entry(op).or_default() += 1;
                result.node_count += 1;
            }
            (2, Field::Bytes(name)) => result.name = utf8(name)?,
            (5, Field::Bytes(weight)) => result.initializers.push(initializer(weight)?),
            (11, Field::Bytes(port)) => result.inputs.push(value_info(port)?),
            (12, Field::Bytes(port)) => result.outputs.push(value_info(port)?),
            _ => {}
        }
        Ok(())
    })
}

pub fn inspect_bytes(bytes: &[u8]) -> Result<OnnxModelReport, String> {
    if bytes.is_empty() || bytes.len() > MAX_MODEL_BYTES {
        return Err("ONNX file is empty or exceeds inspection size limit".into());
    }
    let mut result = OnnxModelReport {
        ir_version: 0,
        graph: OnnxGraphReport::default(),
    };
    let mut found_graph = false;
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.next()? {
        match (number, value) {
            (1, Field::Integer(ir_version)) => result.ir_version = ir_version,
            (7, Field::Bytes(contents)) => {
                if found_graph {
                    return Err("multiple ONNX graphs".into());
                }
                result.graph = graph(contents)?;
                found_graph = true;
            }
            _ => {}
        }
    }
    if !found_graph || result.ir_version == 0 {
        return Err("ONNX model lacks a graph or IR version".into());
    }
    Ok(result)
}

pub fn inspect_file(path: &Path) -> Result<OnnxModelReport, String> {
    let length = fs::metadata(path).map_err(|error| error.to_string())?.len();
    if length == 0 || length > MAX_MODEL_BYTES as u64 {
        return Err("ONNX model size outside inspection bounds".into());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    inspect_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        while value >= 128 {
            bytes.push((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
        bytes.push(value as u8);
        bytes
    }

    fn field(number: u8, data: &[u8]) -> Vec<u8> {
        let mut output = varint(u64::from(number) * 8 + 2);
        output.extend(varint(data.len() as u64));
        output.extend(data);
        output
    }

    fn integer(number: u8, value: u64) -> Vec<u8> {
        let mut output = varint(u64::from(number) * 8);
        output.extend(varint(value));
        output
    }

    #[test]
    fn extracts_graph_nodes_shapes_and_weights() {
        let dim = integer(1, 16);
        let shape = field(1, &dim);
        let mut tensor_type = integer(1, 1);
        tensor_type.extend(field(2, &shape));
        let ty = field(1, &tensor_type);
        let mut input = field(1, b"features");
        input.extend(field(2, &ty));
        let node = field(4, b"MatMul");
        let mut weight = integer(1, 16);
        weight.extend(integer(1, 80));
        weight.extend(integer(2, 1));
        weight.extend(field(8, b"linear.weight"));
        let mut graph = field(1, &node);
        graph.extend(field(2, b"streaming_encoder"));
        graph.extend(field(5, &weight));
        graph.extend(field(11, &input));
        graph.extend(field(12, &input));
        let mut model = integer(1, 8);
        model.extend(field(7, &graph));
        let report = inspect_bytes(&model).unwrap();
        assert_eq!(report.ir_version, 8);
        assert_eq!(report.graph.operators["MatMul"], 1);
        assert_eq!(report.graph.initializers[0].dimensions, [16, 80]);
        assert_eq!(report.graph.inputs[0].dimensions, [Dimension::Fixed(16)]);
    }

    #[test]
    fn rejects_invalid_and_truncated_wire_data() {
        assert!(inspect_bytes(&[]).is_err());
        assert!(inspect_bytes(&[0x0a, 0x08, 0x01]).is_err());
        assert!(inspect_bytes(&[0]).is_err());
        assert!(inspect_bytes(&[0x08, 0x80]).is_err());
    }

    #[test]
    fn supports_symbolic_dimensions() {
        let dimension = field(2, b"num_frames");
        assert_eq!(
            tensor_dimension(&dimension).unwrap(),
            Dimension::Symbol("num_frames".into())
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightExportReport {
    pub tensor_count: usize,
    pub total_bytes: usize,
}

struct EmbeddedWeight<'a> {
    name: String,
    dimensions: Vec<u64>,
    dtype: &'static str,
    data: &'a [u8],
}

fn safetensors_dtype(value: u64) -> Option<(&'static str, usize)> {
    match value {
        1 => Some(("F32", 4)),
        2 => Some(("U8", 1)),
        3 => Some(("I8", 1)),
        5 => Some(("I16", 2)),
        6 => Some(("I32", 4)),
        7 => Some(("I64", 8)),
        9 => Some(("BOOL", 1)),
        10 => Some(("F16", 2)),
        11 => Some(("F64", 8)),
        16 => Some(("BF16", 2)),
        _ => None,
    }
}

fn embedded_weight(bytes: &[u8]) -> Result<EmbeddedWeight<'_>, String> {
    let mut name = String::new();
    let mut dims = Vec::new();
    let mut data_type = 0;
    let mut raw_data = None;
    let mut external = false;
    let mut wire = Wire::new(bytes);
    while let Some((number, value)) = wire.next()? {
        match (number, value) {
            (1, Field::Integer(dim)) => dims.push(dim),
            (1, Field::Bytes(packed)) => {
                let mut values = Wire::new(packed);
                while values.offset < packed.len() {
                    dims.push(values.varint()?);
                }
            }
            (2, Field::Integer(dtype)) => data_type = dtype,
            (8, Field::Bytes(tensor_name)) => name = utf8(tensor_name)?,
            (9, Field::Bytes(data)) => raw_data = Some(data),
            (14, Field::Integer(location)) => external = location == 1,
            _ => {}
        }
    }
    if name.is_empty() || external {
        return Err("initializer has no name or uses external storage".into());
    }
    let (dtype, item_bytes) = safetensors_dtype(data_type)
        .ok_or_else(|| format!("unsupported initializer dtype {data_type} for {name}"))?;
    let expected = dims.iter().try_fold(item_bytes, |size, dim| {
        size.checked_mul(usize::try_from(*dim).map_err(|_| "dimension overflow")?)
            .ok_or("tensor size overflow")
    })?;
    let data = raw_data.ok_or_else(|| format!("initializer {name} does not contain raw_data"))?;
    if data.len() != expected {
        return Err(format!(
            "initializer {name} byte length mismatch: expected {expected}, got {}",
            data.len()
        ));
    }
    Ok(EmbeddedWeight {
        name,
        dimensions: dims,
        dtype,
        data,
    })
}

pub fn export_embedded_initializers(
    model_path: &Path,
    destination: &Path,
) -> Result<WeightExportReport, String> {
    use std::io::{BufWriter, Write};

    let metadata = fs::metadata(model_path).map_err(|error| error.to_string())?;
    if metadata.len() == 0 || metadata.len() > MAX_MODEL_BYTES as u64 {
        return Err("ONNX model size outside export bounds".into());
    }
    let bytes = fs::read(model_path).map_err(|error| error.to_string())?;
    let mut model = Wire::new(&bytes);
    let mut graph_bytes = None;
    while let Some((number, value)) = model.next()? {
        if let (7, Field::Bytes(graph)) = (number, value) {
            if graph_bytes.replace(graph).is_some() {
                return Err("ONNX model contains multiple graphs".into());
            }
        }
    }
    let graph_bytes = graph_bytes.ok_or("ONNX graph missing")?;
    let mut graph = Wire::new(graph_bytes);
    let mut weights = BTreeMap::new();
    while let Some((number, value)) = graph.next()? {
        if let (5, Field::Bytes(tensor)) = (number, value) {
            let weight = embedded_weight(tensor)?;
            if weights.insert(weight.name.clone(), weight).is_some() {
                return Err("duplicate initializer name".into());
            }
        }
    }
    if weights.is_empty() {
        return Err("ONNX graph has no embedded initializers".into());
    }
    let mut header = serde_json::Map::new();
    let mut offset = 0usize;
    for (name, weight) in &weights {
        let end = offset
            .checked_add(weight.data.len())
            .ok_or("tensor offsets overflow")?;
        header.insert(
            name.clone(),
            serde_json::json!({
                "dtype": weight.dtype,
                "shape": weight.dimensions,
                "data_offsets": [offset, end]
            }),
        );
        offset = end;
    }
    let mut header_bytes = serde_json::to_vec(&header).map_err(|error| error.to_string())?;
    while header_bytes.len() % 8 != 0 {
        header_bytes.push(b' ');
    }
    let header_length =
        u64::try_from(header_bytes.len()).map_err(|_| "safetensors header too large")?;
    let target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("cannot create new safetensors output: {error}"))?;
    let mut output = BufWriter::new(target);
    output
        .write_all(&header_length.to_le_bytes())
        .map_err(|error| error.to_string())?;
    output
        .write_all(&header_bytes)
        .map_err(|error| error.to_string())?;
    for weight in weights.values() {
        output
            .write_all(weight.data)
            .map_err(|error| error.to_string())?;
    }
    output.flush().map_err(|error| error.to_string())?;
    Ok(WeightExportReport {
        tensor_count: weights.len(),
        total_bytes: offset,
    })
}

#[cfg(test)]
mod export_tests {
    use super::*;

    fn encode_field(id: u64, bytes: &[u8]) -> Vec<u8> {
        let mut output = vec![(id as u8) * 8 + 2, bytes.len() as u8];
        output.extend(bytes);
        output
    }

    #[test]
    fn rejects_missing_raw_tensor_data() {
        let mut tensor = vec![0x08, 0x02, 0x10, 0x01];
        tensor.extend(encode_field(8, b"x"));
        assert!(embedded_weight(&tensor).is_err());
    }

    #[test]
    fn reads_exact_embedded_float_bytes() {
        let mut tensor = vec![0x08, 0x02, 0x10, 0x01];
        tensor.extend(encode_field(8, b"x"));
        tensor.extend(encode_field(9, &[0, 0, 128, 63, 0, 0, 0, 64]));
        let result = embedded_weight(&tensor).unwrap();
        assert_eq!(result.name, "x");
        assert_eq!(result.dimensions, [2]);
        assert_eq!(result.dtype, "F32");
        assert_eq!(result.data.len(), 8);
    }

    #[test]
    fn exports_valid_safetensors_header_and_preserves_payload() {
        let mut tensor = vec![0x08, 0x02, 0x10, 0x01];
        tensor.extend(encode_field(8, b"weight"));
        let raw = [0u8, 0, 128, 63, 0, 0, 0, 64];
        tensor.extend(encode_field(9, &raw));
        let graph = encode_field(5, &tensor);
        let mut model = vec![0x08, 0x08];
        model.extend(encode_field(7, &graph));
        let directory = std::env::temp_dir().join(format!(
            "zipeek-onnx-export-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("test.onnx");
        let target = directory.join("test.safetensors");
        std::fs::write(&source, &model).unwrap();
        let report = export_embedded_initializers(&source, &target).unwrap();
        assert_eq!(report.tensor_count, 1);
        assert_eq!(report.total_bytes, raw.len());
        let exported = std::fs::read(&target).unwrap();
        let size = u64::from_le_bytes(exported[..8].try_into().unwrap()) as usize;
        let header: serde_json::Value = serde_json::from_slice(&exported[8..8 + size]).unwrap();
        assert_eq!(header["weight"]["dtype"], "F32");
        assert_eq!(header["weight"]["shape"], serde_json::json!([2]));
        assert_eq!(&exported[8 + size..], &raw);
        assert!(export_embedded_initializers(&source, &target).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
