//! Document frontmatter recognition: a leading `===…===` block whose inner
//! content is captured raw (YAML not parsed here). `fence_marker` is the single
//! definition of a fence line; `fence_line` exposes it for the open fence, the
//! close fence, and (once wired) the `at_content_line` / `block()` guards that
//! keep fences at the document start.

use std::ops::Range;

use argdown_core::{Metadata, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, peek, repeat};
use winnow::token::take_while;

use crate::Input;
use crate::text::inline_ws;
use crate::trivia::blank_line;

/// Match one fence line and return the byte range of its `=` run. A fence line
/// is: optional leading whitespace, three or more `=`, optional trailing
/// whitespace, then a line ending or EOF. Backtrackable: fewer than three `=`,
/// or trailing non-whitespace, fails so the line is treated as ordinary content.
fn fence_marker(input: &mut Input<'_>) -> ModalResult<Range<usize>> {
    inline_ws(input)?;
    let (_, span) = take_while(3.., '=').with_span().parse_next(input)?;
    inline_ws(input)?;
    alt((line_ending.void(), eof.void())).parse_next(input)?;
    Ok(span)
}

/// Unit-result wrapper around `fence_marker`: succeeds (consuming the fence
/// line) when the current line is a fence line. Callers use `not(fence_line)` to
/// stop before a fence and `peek(fence_line)` to test without advancing.
pub(crate) fn fence_line(input: &mut Input<'_>) -> ModalResult<()> {
    fence_marker(input)?;
    Ok(())
}

/// One raw frontmatter body line: any line that is neither a fence line nor EOF.
/// Backtracks (consuming nothing) at the closing fence or at EOF so the body
/// `repeat` stops there.
fn body_line(input: &mut Input<'_>) -> ModalResult<()> {
    not(fence_line).parse_next(input)?;
    not(eof).parse_next(input)?;
    till_line_ending.void().parse_next(input)?;
    opt(line_ending).void().parse_next(input)?;
    Ok(())
}

/// Recognize a leading `===…===` frontmatter block. Returns `Metadata` where
/// `raw` is the verbatim body between the fences (line endings preserved, both
/// fence lines excluded) and `span` runs from the first `=` of the open fence to
/// just past the last `=` of the close fence. The open fence is backtrackable
/// (so `opt(frontmatter)` yields `None` on a non-frontmatter document); after it
/// matches, failures are hard `Cut` errors: no closing fence before EOF
/// (unterminated), or content on the line immediately after the close (a missing
/// paragraph break).
pub(crate) fn frontmatter(input: &mut Input<'_>) -> ModalResult<Metadata> {
    let open = fence_marker(input)?;
    let (_, raw): ((), &str) = repeat(0.., body_line).with_taken().parse_next(input)?;
    let close = cut_err(fence_marker).parse_next(input)?;
    cut_err(peek(alt((
        eof.void(),
        blank_line.void(),
        (inline_ws, eof).void(),
    ))))
    .parse_next(input)?;
    Ok(Metadata {
        raw: raw.to_string(),
        span: Span {
            start: open.start,
            end: close.end,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_frontmatter(src: &str) -> Result<Metadata, ()> {
        let mut input = Input::new(src);
        frontmatter(&mut input).map_err(|_| ())
    }

    #[test]
    fn captures_basic_block() {
        let m = run_frontmatter("===\ntitle: X\nauthor: Y\n===\n").unwrap();
        assert_eq!(m.raw, "title: X\nauthor: Y\n");
    }

    #[test]
    fn span_covers_the_whole_fenced_block() {
        let src = "===\ntitle: X\nauthor: Y\n===\n";
        let mut input = Input::new(src);
        let m = frontmatter(&mut input).unwrap();
        assert_eq!(
            &src[m.span.start..m.span.end],
            "===\ntitle: X\nauthor: Y\n==="
        );
    }

    #[test]
    fn empty_body_is_captured() {
        let m = run_frontmatter("===\n===\n").unwrap();
        assert_eq!(m.raw, "");
    }

    #[test]
    fn crlf_body_preserves_line_endings() {
        let src = "===\r\ntitle: X\r\n===\r\n";
        let mut input = Input::new(src);
        let m = frontmatter(&mut input).unwrap();
        assert_eq!(m.raw, "title: X\r\n");
        assert_eq!(&src[m.span.start..m.span.end], "===\r\ntitle: X\r\n===");
    }

    #[test]
    fn four_or_more_equals_is_a_fence() {
        assert!(run_frontmatter("====\na: b\n====\n").is_ok());
    }

    #[test]
    fn indented_fence_is_a_fence() {
        let m = run_frontmatter("  ===\na: b\n  ===\n").unwrap();
        assert_eq!(m.raw, "a: b\n");
    }

    #[test]
    fn non_yaml_body_is_still_captured() {
        // The recognizer never parses YAML; any body text is captured verbatim.
        let m = run_frontmatter("===\nthis is: not: valid: yaml\n===\n").unwrap();
        assert_eq!(m.raw, "this is: not: valid: yaml\n");
    }

    #[test]
    fn two_equals_is_not_a_fence() {
        // `==` opens no frontmatter: fence_marker backtracks, frontmatter errors.
        assert!(run_frontmatter("==\na: b\n==\n").is_err());
    }

    #[test]
    fn unterminated_block_is_an_error() {
        assert!(run_frontmatter("===\ntitle: X\n").is_err());
    }

    #[test]
    fn eof_immediately_after_close_is_ok() {
        // No trailing newline at all after the closing fence.
        assert!(run_frontmatter("===\na: b\n===").is_ok());
    }

    #[test]
    fn content_immediately_after_close_is_an_error() {
        // A blank line (or EOF) must follow the close fence; immediate content
        // on the next line is the missing-paragraph-break hard error.
        assert!(run_frontmatter("===\na: b\n===\nnext paragraph").is_err());
    }
}
