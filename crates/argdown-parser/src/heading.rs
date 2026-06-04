//! ATX headings (`#`–`######`).

use argdown_core::Heading;
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::opt;
use winnow::token::take_while;

use crate::Input;
use crate::inline::scan_line;
use crate::metadata::capture_metadata;
use crate::trivia::strip_trailing_line_comment;

/// Parse one ATX heading: 1–6 `#`, at least one space/tab, then text to EOL.
pub(crate) fn heading(input: &mut Input<'_>) -> ModalResult<Heading> {
    let (level, span) = take_while(1..=6, '#')
        .map(|hashes: &str| hashes.len() as u8)
        .with_span()
        .parse_next(input)?;

    // Consume the whitespace between `#`s and the title as its own step so
    // that `text_span.start` is the absolute offset of `raw[0]`, not the
    // offset of the first space.
    take_while(1.., [' ', '\t']).void().parse_next(input)?;
    let (raw, text_span) = till_line_ending.with_span().parse_next(input)?;

    opt(line_ending).parse_next(input)?;

    let heading_span = span.start..text_span.end;

    // Check for a trailing metadata block on this single line.
    let (_inlines, content_len, meta_open) = scan_line(raw, text_span.start)
        .map_err(|_| winnow::error::ErrMode::Cut(winnow::error::ContextError::new()))?;

    let metadata = match meta_open {
        Some(open) => {
            let m = capture_metadata(raw, text_span.start, open)
                .map_err(|_| winnow::error::ErrMode::Cut(winnow::error::ContextError::new()))?;
            // Only trivia (whitespace or a `//` comment) may follow the
            // closing `}`. Non-trivia text is a hard error.
            let end_in_raw = m.span.end - text_span.start;
            let tail = raw[end_in_raw..].split("//").next().unwrap_or("").trim();
            if !tail.is_empty() {
                return Err(winnow::error::ErrMode::Cut(
                    winnow::error::ContextError::new(),
                ));
            }
            Some(m)
        }
        None => None,
    };

    let text = strip_trailing_line_comment(&raw[..content_len])
        .trim()
        .to_string();

    Ok(Heading {
        level,
        text,
        span: heading_span.into(),
        metadata,
    })
}
