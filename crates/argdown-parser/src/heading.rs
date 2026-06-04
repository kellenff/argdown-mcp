//! ATX headings (`#`–`######`).

use argdown_core::Heading;
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{opt, preceded};
use winnow::token::take_while;

use crate::Input;
use crate::trivia::strip_trailing_line_comment;

/// Parse one ATX heading: 1–6 `#`, at least one space/tab, then text to EOL.
pub(crate) fn heading(input: &mut Input<'_>) -> ModalResult<Heading> {
    let ((level, raw), span) = (
        take_while(1..=6, '#').map(|hashes: &str| hashes.len() as u8),
        preceded(take_while(1.., [' ', '\t']), till_line_ending),
    )
        .with_span()
        .parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok(Heading {
        level,
        text: strip_trailing_line_comment(raw).trim().to_string(),
        span: span.into(),
        metadata: None,
    })
}
