mod pack;
mod paths;
mod unpack;

pub use pack::pack;
pub use unpack::unpack;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
