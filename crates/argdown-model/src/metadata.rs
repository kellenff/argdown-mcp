//! Metadata parsing (Layer B, slice B2).
//!
//! Turns the verbatim raw YAML content the parser already captures in
//! [`argdown_core::Metadata`] into a parsed value tree. Pure and partial —
//! [`parse_metadata`] returns `Result` because the raw content can be invalid
//! YAML. Both element metadata (`{…}`) and document frontmatter
//! (`===…===`) flow through this function; the parser produces the same
//! [`argdown_core::Metadata`] shape for both, so B2 does not distinguish them.

use argdown_core::Metadata;

pub use noyalib::compat::serde_yaml::Value;

/// A metadata parse failure. Carries a human-readable message and the byte
/// offset within the raw content where parsing failed (so callers can point
/// at the failing token in the source).
#[derive(Debug)]
pub struct MetadataError {
    pub message: String,
    pub offset: usize,
}

/// Parse the raw YAML content of a `Metadata` into a `Value` tree.
///
/// Accepts any YAML root: mapping, sequence, scalar, null, or tagged value.
/// Element metadata and document frontmatter both flow through this
/// function — the parser produces the same `Metadata { raw, span }` shape
/// for both, so B2 does not distinguish them.
pub fn parse_metadata(_meta: &Metadata) -> Result<Value, MetadataError> {
    Err(MetadataError {
        message: "parse_metadata: not yet implemented".to_string(),
        offset: 0,
    })
}
