pub mod cors;
pub mod proxy;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;