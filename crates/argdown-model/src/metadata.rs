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
pub fn parse_metadata(meta: &Metadata) -> Result<Value, MetadataError> {
    noyalib::compat::serde_yaml::from_str(&meta.raw).map_err(|e| MetadataError {
        message: e.to_string(),
        offset: e.location().map_or(0, |m| m.index()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_core::{Metadata, Span};

    #[test]
    fn parses_mapping_root() {
        // B2 sees the raw content between the braces; for "{k: v}" the
        // captured Metadata.raw is the string "k: v".
        let meta = Metadata {
            raw: "k: v".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(
            matches!(v, Value::Mapping(_)),
            "expected Value::Mapping, got {v:?}"
        );
    }
}
