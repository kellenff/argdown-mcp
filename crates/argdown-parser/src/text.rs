//! Shared line helpers: continuation lines, normalization, block boundaries,
//! and the "text after a reference" guard.

use std::ops::Range;

use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{digit1, line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, repeat};
use winnow::error::StrContext;
use winnow::token::{one_of, take_while};

use crate::Input;
use crate::trivia::{blank_line, comment_start, heading_marker, strip_trailing_line_comment};

/// Consume run of spaces and tabs (no line breaks).
pub(crate) fn inline_ws(input: &mut Input<'_>) -> ModalResult<()> {
    take_while(0.., [' ', '\t']).void().parse_next(input)
}

/// Match a line that begins (after indentation) with `[` or `<` — i.e. the
/// start of a new statement/argument block.
pub(crate) fn block_head(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), one_of(['[', '<']))
        .void()
        .parse_next(input)
}

/// Match a line that begins (after indentation) with a relation operator: its
/// first non-space char is `+`, `-`, `_`, or `>`, or `<` followed by `+`/`-`/`_`.
/// Lets a continuation line (and `finish_reference`) stop before a following
/// relation line instead of swallowing it as text.
pub(crate) fn relation_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (
        take_while(0.., [' ', '\t']),
        alt((
            one_of(['+', '-', '_', '>']).void(),
            ('<', one_of(['+', '-', '_'])).void(),
        )),
    )
        .void()
        .parse_next(input)
}

/// Match a line that begins (after indentation) with a numbered marker
/// `( digits )` — the start of a PCS statement. Lets a continuation line stop
/// before the next numbered statement instead of swallowing it as text.
pub(crate) fn pcs_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), '(', digit1, ')')
        .void()
        .parse_next(input)
}

/// Succeeds (consuming nothing) when the cursor is at the start of a plain
/// content line — not EOF, blank, a heading, a comment, or a new block.
/// This is the shared precondition for `content_line` and the
/// reference-continuation check in `finish_reference`.
fn at_content_line(input: &mut Input<'_>) -> ModalResult<()> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
        not(block_head),
        not(relation_marker),
        not(pcs_marker),
    )
        .void()
        .parse_next(input)
}

/// One continuation content line: not EOF, blank, a heading, a comment, or a
/// new block. Returns the raw line (no line ending) and its byte span.
pub(crate) fn content_line<'s>(input: &mut Input<'s>) -> ModalResult<(&'s str, Range<usize>)> {
    at_content_line.parse_next(input)?;
    let (line, span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok((line, span))
}

/// Strip trailing line comments, trim, drop empties, join with a single space.
pub(crate) fn normalize_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    let mut parts: Vec<&'a str> = Vec::new();
    for line in lines {
        let trimmed = strip_trailing_line_comment(line).trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}

/// Read a definition body: the remainder of the current line plus continuation
/// content lines. Returns the normalized text and the body's end byte offset.
pub(crate) fn definition_body(input: &mut Input<'_>) -> ModalResult<(String, usize)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);
    let text = normalize_lines(std::iter::once(first).chain(rest.iter().map(|(line, _)| *line)));
    Ok((text, end))
}

/// Called right after a reference's closing bracket and `inline_ws`. Allows an
/// optional trailing line comment, then requires end-of-line/EOF, then forbids
/// a plain-text continuation line. Emits a hard `cut_err` at the offending text.
pub(crate) fn finish_reference(input: &mut Input<'_>) -> ModalResult<()> {
    opt(("//", till_line_ending).void()).parse_next(input)?;
    // No free text may follow the closing bracket on the same line.
    cut_err(
        alt((line_ending.void(), eof.void())).context(StrContext::Label(
            "end of reference line (text is not allowed after a reference)",
        )),
    )
    .parse_next(input)?;
    // A reference also cannot be continued by a plain-text line on the next line.
    cut_err(not(at_content_line).context(StrContext::Label("text content after a reference")))
        .parse_next(input)?;
    Ok(())
}
