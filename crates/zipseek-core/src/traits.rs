use anyhow::Result;

pub trait ProtobufDeserialize {
    fn decode_from_slice(data: &[u8]) -> Result<Self> where Self: Sized;
}