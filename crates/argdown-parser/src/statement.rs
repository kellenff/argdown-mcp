//! Statements: plain text, titled definitions (`[T]: x`), and references (`[T]`).

use std::ops::Range;

use argdown_core::{Span, Statement};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{alt, delimited, eof, not, opt};
use winnow::token::take_till;

use crate::Input;
use crate::text::{body_lines, definition_body, finish_reference, inline_ws, process_body};
use crate::trivia::{blank_line, comment_start, heading_marker};

/// Parse one statement: a bracketed definition/reference, or plain text.
pub(crate) fn statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    alt((bracketed_statement, plain_statement)).parse_next(input)
}

/// `[Title]: text` (definition) or `[Title]` (reference). Once `[Title]` is
/// consumed the branch is committed, so trailing text is a hard error.
fn bracketed_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    let (title, span) = statement_title.parse_next(input)?;
    if opt(':').parse_next(input)?.is_some() {
        let (text, end, inlines, metadata) = definition_body(input)?;
        Ok(Statement {
            title: Some(title),
            text,
            is_reference: false,
            span: Span {
                start: span.start,
                end,
            },
            inlines,
            metadata,
        })
    } else {
        inline_ws.parse_next(input)?;
        finish_reference(input)?;
        Ok(Statement {
            title: Some(title),
            text: String::new(),
            is_reference: true,
            span: span.into(),
            inlines: vec![],
            metadata: None,
        })
    }
}

/// `[ title ]` — title trimmed; fails (backtracks) if there is no closing `]`
/// on the same line, so malformed brackets fall through to plain text.
fn statement_title(input: &mut Input<'_>) -> ModalResult<(String, Range<usize>)> {
    delimited('[', take_till(0.., (']', '\r', '\n')), ']')
        .map(|title: &str| title.trim().to_string())
        .with_span()
        .parse_next(input)
}

/// A plain statement: one or more content lines of free text, normalized. The
/// body extent is brace-aware so a trailing multi-line `{…}` metadata block is
/// read whole.
fn plain_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
    )
        .parse_next(input)?;
    let body = body_lines(input)?;
    // The precondition above guarantees a first line, so the body is non-empty.
    let span_start = body.lines[0].1.start;
    let end = body.end;
    let (text, inlines, metadata) = process_body(&body)?;
    Ok(Statement {
        title: None,
        text,
        is_reference: false,
        span: Span {
            start: span_start,
            end,
        },
        inlines,
        metadata,
    })
}
