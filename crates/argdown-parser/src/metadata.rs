//! Trailing `{yaml}` metadata block recognition. Captures the raw inner content
//! and source span of a balanced `{…}` block; the YAML is not parsed here.

use argdown_core::{Metadata, Span};

/// A metadata recognition failure: an unterminated `{` block.
#[derive(Debug)]
pub(crate) struct MetaError;

/// Capture the balanced `{…}` block in `src` that starts at byte index `open`
/// (`src[open]` must be `{`). `base` is the absolute source offset of `src[0]`.
/// Brace depth is tracked while skipping over quoted strings (`"…"`, `'…'`), so
/// braces inside quotes don't miscount; the block may span multiple lines.
/// Returns the metadata (`raw` = inner content verbatim, `span` = the whole
/// block) or `MetaError` if there is no matching `}` before the end of `src`.
pub(crate) fn capture_metadata(src: &str, base: usize, open: usize) -> Result<Metadata, MetaError> {
    let bytes = src.as_bytes();
    debug_assert_eq!(bytes[open], b'{');
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut i = open;
    while i < src.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2; // escaped char inside a double-quoted string
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Metadata {
                            raw: src[open + 1..i].to_string(),
                            span: Span {
                                start: base + open,
                                end: base + i + 1,
                            },
                        });
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    Err(MetaError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_single_line_block() {
        let src = "{k: v}";
        let m = capture_metadata(src, 100, 0).unwrap();
        assert_eq!(m.raw, "k: v");
        assert_eq!(
            m.span,
            argdown_core::Span {
                start: 100,
                end: 106
            }
        );
    }

    #[test]
    fn captures_nested_and_quoted_braces() {
        assert_eq!(
            capture_metadata("{a: {b: 1}}", 0, 0).unwrap().raw,
            "a: {b: 1}"
        );
        assert_eq!(
            capture_metadata("{n: \"a } b\"}", 0, 0).unwrap().raw,
            "n: \"a } b\""
        );
    }

    #[test]
    fn captures_multi_line_block() {
        let src = "{\n  a: b\n  c: d\n}";
        let m = capture_metadata(src, 0, 0).unwrap();
        assert_eq!(m.raw, "\n  a: b\n  c: d\n");
        assert_eq!(m.span.end, src.len());
    }

    #[test]
    fn unterminated_block_errors() {
        assert!(capture_metadata("{a: b", 0, 0).is_err());
    }
}
