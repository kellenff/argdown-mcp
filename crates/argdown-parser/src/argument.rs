//! Arguments: titled definitions (`<T>: desc`) and references (`<T>`).

use std::ops::Range;

use argdown_core::{Argument, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{delimited, opt};
use winnow::token::take_till;

use crate::Input;
use crate::text::{definition_body, finish_reference_with_metadata, inline_ws};

/// `<Title>: description` (definition) or `<Title>` (reference). Once
/// `<Title>` is consumed the branch is committed; trailing text is an error.
pub(crate) fn argument(input: &mut Input<'_>) -> ModalResult<Argument> {
    let (title, span) = argument_title.parse_next(input)?;
    if opt(':').parse_next(input)?.is_some() {
        let (description, end, inlines, metadata) = definition_body(input)?;
        Ok(Argument {
            title,
            description,
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
        let metadata = finish_reference_with_metadata(input)?;
        let end = metadata.as_ref().map_or(span.end, |m| m.span.end);
        Ok(Argument {
            title,
            description: String::new(),
            is_reference: true,
            span: Span {
                start: span.start,
                end,
            },
            inlines: vec![],
            metadata,
        })
    }
}

/// `< title >` — title trimmed; fails (backtracks) without a closing `>` on
/// the same line, so an unterminated `<` falls through to plain text.
fn argument_title(input: &mut Input<'_>) -> ModalResult<(String, Range<usize>)> {
    delimited('<', take_till(0.., ('>', '\r', '\n')), '>')
        .map(|title: &str| title.trim().to_string())
        .with_span()
        .parse_next(input)
}
