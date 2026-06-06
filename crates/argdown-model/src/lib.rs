//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing, B3 provides statement equivalence classes.

mod metadata;
mod sections;
mod statements;

pub use metadata::{MetadataError, Value, parse_metadata};
pub use sections::{Section, SectionId, Sections, build_sections};
pub use statements::{Statement, StatementConflict, StatementId, Statements, build_statements};
