//! Winnow-based parser for the Argdown format.
//!
//! Turns source text into an [`argdown_core::Document`]. The grammar is a
//! stub for now and will be implemented incrementally.

use argdown_core::{Document, Error};

/// Parse Argdown source text into a [`Document`].
///
/// Currently a stub: it accepts any input and returns an empty document.
/// The real winnow grammar will replace this body.
pub fn parse(_source: &str) -> Result<Document, Error> {
    Ok(Document::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_input_yields_empty_document() {
        assert_eq!(parse(""), Ok(argdown_core::Document::default()));
    }
}
