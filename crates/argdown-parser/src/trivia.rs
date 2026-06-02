//! Whitespace, blank lines, heading-marker detection, and comment helpers.

use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::line_ending;
use winnow::token::{one_of, take_while};

use crate::Input;

/// Skip inter-block trivia: runs of whitespace and line breaks.
pub(crate) fn skip_trivia(input: &mut Input<'_>) -> ModalResult<()> {
    take_while(0.., [' ', '\t', '\r', '\n'])
        .void()
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

/// Remove a trailing `// …` line comment from raw line text.
pub(crate) fn strip_trailing_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}
