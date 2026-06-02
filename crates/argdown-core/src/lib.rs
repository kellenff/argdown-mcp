//! Core domain types for Argdown documents.
//!
//! These are the precise types the parser produces and the rest of the
//! program is written against. The model will grow as the grammar is
//! implemented; for now it is a minimal placeholder.

/// A parsed Argdown document.
///
/// Empty for now; the parser will populate fields as the grammar is built.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {}

/// Errors produced while turning source text into a [`Document`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The parser could not interpret the input.
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(message) => write!(f, "parse error: {message}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_parse_message() {
        let err = Error::Parse("unexpected token".to_string());
        assert_eq!(err.to_string(), "parse error: unexpected token");
    }

    #[test]
    fn document_default_is_constructible() {
        let _doc = Document::default();
    }
}
