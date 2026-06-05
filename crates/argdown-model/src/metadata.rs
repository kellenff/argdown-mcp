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

    #[test]
    fn parses_scalar_string_root() {
        let meta = Metadata {
            raw: "hello".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::String(ref s) if s == "hello"));
    }

    #[test]
    fn parses_scalar_int_root() {
        let meta = Metadata {
            raw: "42".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Number(_)));
    }

    #[test]
    fn parses_scalar_bool_root() {
        let meta = Metadata {
            raw: "true".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn parses_scalar_null_root() {
        let meta = Metadata {
            raw: "null".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn parses_sequence_root() {
        let meta = Metadata {
            raw: "[a, b, c]".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        let Value::Sequence(seq) = v else {
            panic!("expected Value::Sequence");
        };
        assert_eq!(seq.len(), 3);
    }

    #[test]
    fn parses_mapping_with_multiple_entries() {
        let meta = Metadata {
            raw: "k: v\nn: 1".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parses_nested_mapping() {
        let meta = Metadata {
            raw: "a:\n  b: c".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        // Outer is a mapping with one key "a".
        let Value::Mapping(map) = v else {
            panic!("expected outer Value::Mapping");
        };
        assert_eq!(map.len(), 1);
        // Inner value for "a" is itself a mapping with one key "b".
        let inner = map.into_iter().next().unwrap().1;
        let Value::Mapping(inner_map) = inner else {
            panic!("expected inner Value::Mapping");
        };
        assert_eq!(inner_map.len(), 1);
    }

    #[test]
    fn empty_raw_is_null() {
        // A frontmatter with no body: `===\n===`. Empty input is valid
        // YAML (the YAML 1.2 null document), so parse_metadata returns
        // Ok(Value::Null) rather than an error.
        let meta = Metadata {
            raw: "".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn invalid_yaml_is_an_error() {
        // Mismatched indentation.
        let meta = Metadata {
            raw: "a: b\n  c: d".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let err = parse_metadata(&meta).unwrap_err();
        // Offset is in [0, raw.len()]. The lib may report 0 if it cannot
        // localize the failure; that's still a valid (in-range) offset.
        assert!(err.offset <= meta.raw.len());
    }

    #[test]
    fn error_offset_within_raw() {
        // For a raw with a known failure point, the offset should be inside
        // the raw (not a global document offset). We don't pin to a specific
        // byte index because the lib's exact localization is an
        // implementation detail; we only check the coordinate space.
        let raw = "good: 1\n  bad_indent: oops".to_string();
        let meta = Metadata {
            raw: raw.clone(),
            span: Span {
                start: 1000,
                end: 1000 + raw.len(),
            },
        };
        let err = parse_metadata(&meta).unwrap_err();
        assert!(
            err.offset <= raw.len(),
            "offset {} should be ≤ raw.len() {} (offset must be in the raw coordinate space, not the global document space)",
            err.offset,
            raw.len()
        );
    }

    #[test]
    fn element_metadata_roundtrip() {
        // Parse a heading with inline `{k: v}` metadata (the parser
        // recognizes trailing metadata on the same line as `# Title`).
        // Capture the raw from the AST; parse it back and confirm it's a
        // one-entry mapping.
        let doc = argdown_parser::parse("# Top {k: v}").unwrap();
        let heading = match &doc.blocks[0] {
            argdown_core::Block::Heading(h) => h,
            other => panic!("expected Heading, got {other:?}"),
        };
        let meta = heading
            .metadata
            .as_ref()
            .expect("heading should have metadata");
        let v = parse_metadata(meta).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn frontmatter_roundtrip() {
        // Parse a document with leading `===…===` frontmatter containing a
        // `title: X` mapping; capture the raw from Document.frontmatter;
        // parse it back and confirm the mapping has a `title` key.
        let doc = argdown_parser::parse("===\ntitle: X\nauthor: Y\n===\n\n# Top").unwrap();
        let fm = doc
            .frontmatter
            .as_ref()
            .expect("document should have frontmatter");
        let v = parse_metadata(fm).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert!(map.contains_key("title"));
    }
}
