//! Core domain types for Argdown documents.
//!
//! The syntax-tree types the parser produces and the rest of the program is
//! written against. Grows as the grammar is implemented.

mod ast;
mod error;

pub use ast::{Block, Document, Heading, Span, Statement};
pub use error::Error;
