//! Shared line helpers: continuation lines, normalization, block boundaries,
//! and the "text after a reference" guard.

use std::ops::Range;

use argdown_core::{Inline, Metadata};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{digit1, line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, peek};
use winnow::error::{ContextError, ErrMode, StrContext};
use winnow::token::{literal, one_of, take_while};

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

/// A raw body line: the slice (no line ending) and its absolute byte span.
pub(crate) type BodyLine<'s> = (&'s str, Range<usize>);

/// One continuation content line: not EOF, blank, a heading, a comment, or a
/// new block. Returns the raw line (no line ending) and its byte span.
pub(crate) fn content_line<'s>(input: &mut Input<'s>) -> ModalResult<(&'s str, Range<usize>)> {
    at_content_line.parse_next(input)?;
    let (line, span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok((line, span))
}

/// Net `{` minus `}` in `line`, ignoring braces inside quotes (and inside a
/// double-quoted string, treating `\` as escaping the next byte). Mirrors the
/// quote/escape rules of `capture_metadata` — `\` is *not* an escape outside
/// quotes — so the body extent and the block scanner agree on where a `{…}`
/// block opens and closes.
fn brace_delta(line: &str) -> isize {
    let bytes = line.as_bytes();
    let mut delta = 0isize;
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < line.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2;
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'{' => delta += 1,
                b'}' => delta -= 1,
                _ => {}
            },
        }
        i += 1;
    }
    delta
}

/// Read continuation lines after the first body line. Normally these are
/// `content_line`s (stopping at a blank/marker/EOF), but while a metadata `{…}`
/// block is open (cumulative brace depth > 0) we consume raw lines
/// unconditionally so the block can close. `open_depth` is the first line's
/// brace delta.
fn body_continuation<'s>(
    input: &mut Input<'s>,
    open_depth: isize,
) -> ModalResult<Vec<BodyLine<'s>>> {
    let mut depth = open_depth;
    let mut lines: Vec<BodyLine> = Vec::new();
    loop {
        if depth > 0 {
            // Inside a metadata block: take the next raw line whatever it is.
            if eof::<_, ContextError>.parse_peek(*input).is_ok() {
                // Unterminated block — let the body scanner surface the error.
                return Ok(lines);
            }
            let (line, span) = till_line_ending.with_span().parse_next(input)?;
            opt(line_ending).parse_next(input)?;
            depth += brace_delta(line);
            lines.push((line, span));
        } else {
            match opt(content_line).parse_next(input)? {
                Some((line, span)) => {
                    depth += brace_delta(line);
                    lines.push((line, span));
                }
                None => return Ok(lines),
            }
        }
    }
}

/// The result of reading a definition/plain-statement body: the body lines (each
/// with its absolute byte span), the verbatim contiguous body source from the
/// first line's start through the last body line, and the body's end offset.
/// `src` together with `lines[0].1.start` lets the metadata block be sliced from
/// the real source — preserving the original line endings (`\r\n` or `\n`).
pub(crate) struct Body<'s> {
    pub lines: Vec<BodyLine<'s>>,
    pub src: &'s str,
    pub end: usize,
}

/// Read all body lines for a definition/plain statement: the first line plus a
/// brace-aware run of continuation lines. Captures the verbatim body source via
/// winnow's taken-slice so the line endings are preserved exactly.
pub(crate) fn body_lines<'s>(input: &mut Input<'s>) -> ModalResult<Body<'s>> {
    let (lines, src) = read_body_lines.with_taken().parse_next(input)?;
    let end = lines.last().map_or(0, |(_, span)| span.end);
    Ok(Body { lines, src, end })
}

/// Inner body reader: the first line plus the brace-aware continuation run.
/// Wrapped by `body_lines` in `with_taken` to recover the verbatim source.
fn read_body_lines<'s>(input: &mut Input<'s>) -> ModalResult<Vec<BodyLine<'s>>> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest = body_continuation(input, brace_delta(first))?;
    let mut lines = Vec::with_capacity(rest.len() + 1);
    lines.push((first, first_span));
    lines.extend(rest);
    Ok(lines)
}

/// Inline-scan each body line and locate a top-level metadata `{`. When found,
/// the block is captured from the verbatim contiguous body source (it may run
/// into later lines; slicing the real source preserves the original line endings
/// and keeps every offset absolute). Returns the normalized text (pre-metadata
/// content), the inlines, and any metadata. Only trivia (whitespace or a `//`
/// comment) may follow the closing `}`, else a hard error; an unterminated block
/// is also a hard error.
pub(crate) fn process_body(body: &Body) -> ModalResult<(String, Vec<Inline>, Option<Metadata>)> {
    let mut inlines = Vec::new();
    let mut contents: Vec<&str> = Vec::new();
    let mut metadata: Option<Metadata> = None;
    // Absolute offset of `body.src[0]` — the body's verbatim source slice.
    let base = body.lines.first().map_or(0, |(_, span)| span.start);
    for (line, span) in &body.lines {
        match scan_line(line, span.start) {
            Ok((mut found, content_len, meta_open)) => {
                inlines.append(&mut found);
                contents.push(&line[..content_len]);
                if let Some(open) = meta_open {
                    // `open` is relative to this line's start (== span.start),
                    // which equals its offset into the contiguous body source.
                    let block_open = span.start - base + open;
                    let m = capture_metadata(body.src, base, block_open)
                        .map_err(|_| ErrMode::<ContextError>::Cut(ContextError::new()))?;
                    // Only trivia may follow the closing `}`.
                    let after = m.span.end - base;
                    let tail = body.src[after..].split("//").next().unwrap_or("").trim();
                    if !tail.is_empty() {
                        return Err(ErrMode::Cut(ContextError::new()));
                    }
                    metadata = Some(m);
                    break;
                }
            }
            Err(_) => return Err(ErrMode::Cut(ContextError::new())),
        }
    }
    Ok((normalize_contents(contents), inlines, metadata))
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
/// lines (brace-aware, so a multi-line `{…}` block is read whole). Returns the
/// normalized text, the body's end offset, the inlines, and any trailing
/// `{…}` metadata.
pub(crate) fn definition_body(
    input: &mut Input<'_>,
) -> ModalResult<(String, usize, Vec<Inline>, Option<Metadata>)> {
    let body = body_lines(input)?;
    let end = body.end;
    let (text, inlines, metadata) = process_body(&body)?;
    Ok((text, end, inlines, metadata))
}

/// Called right after a reference's closing bracket and `inline_ws`. Captures an
/// optional trailing `{…}` metadata block (the only non-comment text allowed
/// after a reference), then enforces the reference end-of-line/continuation
/// guard. Returns the captured metadata, if any. A reference followed by plain
/// text (anything other than a metadata block or a `//` comment) is still a hard
/// error.
pub(crate) fn finish_reference_with_metadata(
    input: &mut Input<'_>,
) -> ModalResult<Option<Metadata>> {
    let metadata = reference_metadata(input)?;
    finish_reference(input)?;
    Ok(metadata)
}

/// If the rest of the current line opens a top-level metadata `{`, capture the
/// block and consume it (leaving the cursor at the trailing trivia/comment for
/// `finish_reference`). Only trivia may precede the `{`; only trivia or a `//`
/// comment may follow the closing `}`. Returns `None` when no metadata block is
/// present, leaving the input untouched.
fn reference_metadata(input: &mut Input<'_>) -> ModalResult<Option<Metadata>> {
    let (line, span) = peek(till_line_ending.with_span()).parse_next(input)?;
    let open = match find_top_level_brace(line) {
        Some(open) => open,
        None => return Ok(None),
    };
    // Only trivia may sit between the reference and the metadata opener.
    if !line[..open].trim().is_empty() {
        return Err(ErrMode::Cut(ContextError::new()));
    }
    let m = capture_metadata(line, span.start, open)
        .map_err(|_| ErrMode::<ContextError>::Cut(ContextError::new()))?;
    // Consume the line up to and including the closing `}`.
    // `consumed` is a byte length; use `literal` to advance by that exact byte
    // slice rather than `take(n)` which would advance n *characters* and
    // over-consume when the metadata block contains multibyte UTF-8 chars.
    let consumed = m.span.end - span.start;
    literal(&line[..consumed]).void().parse_next(input)?;
    inline_ws.parse_next(input)?;
    Ok(Some(m))
}

/// Byte index of the first unescaped (top-level) `{` in `s`, or `None`. `\{` is
/// literal, matching the metadata scanner's escape rule.
fn find_top_level_brace(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'{' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

/// Called right after a reference's closing bracket and `inline_ws` (and any
/// trailing metadata block). Allows an optional trailing line comment, then
/// requires end-of-line/EOF, then forbids a plain-text continuation line. Emits
/// a hard `cut_err` at the offending text.
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
