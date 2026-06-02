//! Plain and titled statements, possibly spanning multiple wrapped lines.

use std::ops::Range;

use argdown_core::{Span, Statement};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{eof, not, opt, repeat};

use crate::Input;
use crate::trivia::{blank_line, heading_marker, strip_trailing_line_comment};

/// Parse one statement: one or more consecutive content lines, normalized.
pub(crate) fn statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    let lines: Vec<(&str, Range<usize>)> = repeat(1.., content_line).parse_next(input)?;
    let start = lines.first().expect("repeat(1..) yields >= 1 line").1.start;
    let end = lines.last().expect("repeat(1..) yields >= 1 line").1.end;

    let cleaned: Vec<&str> = lines
        .iter()
        .map(|(line, _)| strip_trailing_line_comment(line))
        .collect();

    let (title, first_rest) = split_title(cleaned[0]);

    let mut parts: Vec<&str> = Vec::new();
    let first = first_rest.trim();
    if !first.is_empty() {
        parts.push(first);
    }
    for line in &cleaned[1..] {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    Ok(Statement {
        title,
        text: parts.join(" "),
        span: Span { start, end },
    })
}

/// One content line: not EOF, not blank, not a heading. Returns the raw line
/// (without its line ending) and the byte span of that text.
fn content_line<'s>(input: &mut Input<'s>) -> ModalResult<(&'s str, Range<usize>)> {
    (not(eof), not(blank_line), not(heading_marker)).parse_next(input)?;
    let (line, span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok((line, span))
}

/// Split a leading `[Title]:` off a line. A bare `[…]` without `]:` is plain
/// text (statement references arrive in increment A2).
fn split_title(line: &str) -> (Option<String>, &str) {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix('[')
        && let Some(close) = rest.find("]:")
    {
        let title = rest[..close].trim().to_string();
        return (Some(title), &rest[close + 2..]);
    }
    (None, line)
}
