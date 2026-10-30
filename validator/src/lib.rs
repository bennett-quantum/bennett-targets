//! Independent ISA 0.8 target validation and canonical identities.
pub mod json;
pub mod expr;
pub mod target;
pub use target::Target;
pub fn validate(s: &str) -> Result<(), String> { Target::load(s).map(|_|()) }
pub fn canon(s: &str) -> Result<(String,String),String> {
    let v=json::parse(s)?; let bytes=json::canonical(&v)?;
    let hash=json::hash(bytes.as_bytes()); Ok((bytes,hash))
}
