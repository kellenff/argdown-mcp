//! Arguments: titled definitions (`<T>: desc`) and references (`<T>`).

use std::ops::Range;

use argdown_core::{Argument, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{delimited, opt};
use winnow::token::take_till;

use crate::Input;
use crate::text::{definition_body, finish_reference, inline_ws};

/// `<Title>: description` (definition) or `<Title>` (reference). Once
/// `<Title>` is consumed the branch is committed; trailing text is an error.
pub(crate) fn argument(input: &mut Input<'_>) -> ModalResult<Argument> {
    let (title, span) = argument_title.parse_next(input)?;
    if opt(':').parse_next(input)?.is_some() {
        let (description, end) = definition_body(input)?;
        Ok(Argument {
            title,
            description,
            is_reference: false,
            span: Span {
                start: span.start,
                end,
            },
        })
    } else {
        inline_ws.parse_next(input)?;
        finish_reference(input)?;
        Ok(Argument {
            title,
            description: String::new(),
            is_reference: true,
            span: span.into(),
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
