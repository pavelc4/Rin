pub mod types;
pub mod index;
pub mod resolver;
pub mod extract;
pub mod manager;
pub const DEFAULT_PREFIX: &str = "/data/data/com.rin/files";

#[cfg(feature = "android")]
pub mod android;
