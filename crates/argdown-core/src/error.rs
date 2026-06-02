//! Error type for Argdown parsing.

/// A parse failure located at a byte offset in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_message_and_offset() {
        let err = Error {
            message: "boom".to_string(),
            offset: 7,
        };
        assert_eq!(err.to_string(), "parse error at byte 7: boom");
    }
}
