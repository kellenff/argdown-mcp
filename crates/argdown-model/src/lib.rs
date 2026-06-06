//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing, B3 provides statement equivalence classes, B4a provides
//! argument equivalence classes.

mod arguments;
mod metadata;
mod sections;
mod statements;

pub use arguments::{Argument, ArgumentConflict, ArgumentId, Arguments, build_arguments};
pub use metadata::{MetadataError, Value, parse_metadata};
pub use sections::{Section, SectionId, Sections, build_sections};
pub use statements::{Statement, StatementConflict, StatementId, Statements, build_statements};
