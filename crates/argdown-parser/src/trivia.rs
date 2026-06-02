//! Whitespace, blank lines, heading-marker detection, and comments.

use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, repeat};
use winnow::token::{one_of, take_until, take_while};

use crate::Input;

/// Skip inter-block trivia, line by line: blank lines and comment lines (each
/// possibly indented) and bare line endings are consumed whole, but the leading
/// indent of a content-bearing line is left for the block parser to measure.
///
/// A content line makes both branches fail at/after the leading whitespace, and
/// `alt` backtracks (restoring the consumed indent), so the cursor rests at
/// column 0 of that line. Top-level elements sit at indent 0, so heading,
/// statement, and argument parsing are unchanged; `relation` measures the
/// indent it needs.
pub(crate) fn skip_trivia(input: &mut Input<'_>) -> ModalResult<()> {
    let _: () = repeat(
        0..,
        alt((
            (take_while(0.., [' ', '\t']), line_ending).void(),
            (take_while(0.., [' ', '\t']), comment).void(),
        )),
    )
    .parse_next(input)?;
    Ok(())
}

/// Consume one comment: line (`// …`), block (`/* … */`), or HTML
/// (`<!-- … -->`). Block and HTML forms may span multiple lines. Fails (with
/// the cursor at the opener) if a block/HTML comment is never closed.
pub(crate) fn comment(input: &mut Input<'_>) -> ModalResult<()> {
    alt((
        ("//", till_line_ending).void(),
        ("/*", take_until(0.., "*/"), "*/").void(),
        ("<!--", take_until(0.., "-->"), "-->").void(),
    ))
    .parse_next(input)
}

/// Match a blank line (only whitespace, then a line ending).
pub(crate) fn blank_line(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), line_ending)
        .void()
        .parse_next(input)
}

/// Match the start of an ATX heading: 1–6 `#` followed by a space or tab.
pub(crate) fn heading_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(1..=6, '#'), one_of([' ', '\t']))
        .void()
        .parse_next(input)
}

/// Match the start of a comment at the beginning of a line (after optional
/// indentation).
pub(crate) fn comment_start(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), alt(("//", "/*", "<!--")))
        .void()
        .parse_next(input)
}

/// Remove a trailing `// …` line comment from raw line text.
pub(crate) fn strip_trailing_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}
