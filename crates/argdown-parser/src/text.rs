//! Shared line helpers: continuation lines, normalization, block boundaries,
//! and the "text after a reference" guard.

use std::ops::Range;

use argdown_core::{Inline, Metadata};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{digit1, line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, repeat};
use winnow::error::{ContextError, ErrMode, StrContext};
use winnow::token::{one_of, take_while};

use crate::Input;
use crate::inline::scan_line;
use crate::metadata::capture_metadata;
use crate::trivia::{blank_line, comment_start, heading_marker};

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

/// Scan one raw body line (`text`, absolute start `base`); append its inlines to
/// `out` and return the content slice (comment- and metadata-stripped) for
/// normalization. If the line opens a single-line metadata `{…}` block, capture
/// it into `meta` (only trivia — whitespace or a `//` comment — may follow the
/// closing `}`, else a hard error).
pub(crate) fn body_line<'s>(
    text: &'s str,
    base: usize,
    out: &mut Vec<Inline>,
    meta: &mut Option<Metadata>,
) -> ModalResult<&'s str> {
    match scan_line(text, base) {
        Ok((mut inlines, content_len, meta_open)) => {
            out.append(&mut inlines);
            if let Some(open) = meta_open {
                match capture_metadata(text, base, open) {
                    Ok(m) => {
                        let end_in_line = m.span.end - base;
                        // Only trivia may follow the closing `}` on this line.
                        let tail = text[end_in_line..].trim_start();
                        if !(tail.is_empty() || tail.starts_with("//")) {
                            return Err(ErrMode::Cut(ContextError::new()));
                        }
                        *meta = Some(m);
                    }
                    Err(_) => return Err(ErrMode::Cut(ContextError::new())),
                }
            }
            Ok(&text[..content_len])
        }
        Err(_) => Err(ErrMode::Cut(ContextError::new())),
    }
}

/// Trim each content slice, drop empties, join with a single space.
pub(crate) fn normalize_contents<'a>(contents: impl IntoIterator<Item = &'a str>) -> String {
    let mut parts: Vec<&'a str> = Vec::new();
    for c in contents {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}

/// Read a definition body: remainder of the current line plus continuation
/// lines. Returns the normalized text, the body's end offset, the inlines, and
/// any single-line trailing `{…}` metadata.
pub(crate) fn definition_body(
    input: &mut Input<'_>,
) -> ModalResult<(String, usize, Vec<Inline>, Option<Metadata>)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let mut inlines = Vec::new();
    let mut metadata: Option<Metadata> = None;
    let mut contents: Vec<&str> = Vec::new();
    contents.push(body_line(
        first,
        first_span.start,
        &mut inlines,
        &mut metadata,
    )?);
    for (line, span) in &rest {
        contents.push(body_line(line, span.start, &mut inlines, &mut metadata)?);
    }
    let text = normalize_contents(contents);
    Ok((text, end, inlines, metadata))
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
