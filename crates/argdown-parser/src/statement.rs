//! Statements: plain text, titled definitions (`[T]: x`), and references (`[T]`).

use std::ops::Range;

use argdown_core::{Inline, Span, Statement};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, delimited, eof, not, opt, repeat};
use winnow::token::take_till;

use crate::Input;
use crate::text::{body_line, content_line, definition_body, finish_reference, inline_ws, normalize_contents};
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
        let (text, end, inlines) = definition_body(input)?;
        Ok(Statement {
            title: Some(title),
            text,
            is_reference: false,
            span: Span {
                start: span.start,
                end,
            },
            inlines,
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

/// A plain statement: one or more content lines of free text, normalized.
fn plain_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
    )
        .parse_next(input)?;
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);

    let mut inlines: Vec<Inline> = Vec::new();
    let mut contents: Vec<&str> = Vec::new();
    contents.push(body_line(first, first_span.start, &mut inlines)?);
    for (line, span) in &rest {
        contents.push(body_line(line, span.start, &mut inlines)?);
    }
    let text = normalize_contents(contents);
    Ok(Statement {
        title: None,
        text,
        is_reference: false,
        span: Span {
            start: first_span.start,
            end,
        },
        inlines,
    })
}
