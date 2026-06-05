//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly.

mod sections;

pub use sections::{Section, SectionId, Sections, build_sections};
