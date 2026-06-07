//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing, B3 provides statement equivalence classes, B4a provides
//! argument equivalence classes, B4b resolves PCS structure and assembles the
//! complete `Model` aggregate, B5 resolves dialectical relation edges, B6a
//! provides the tag registry.

mod arguments;
mod dung;
mod export;
mod import;
mod metadata;
mod model;
mod sections;
mod statements;
mod tags;

pub use arguments::{Argument, ArgumentConflict, ArgumentId, Arguments, build_arguments};
pub use dung::{
    AfMetadata, Algorithm, ArgumentationFramework, GroundedLabelling, Label, Labeling, Semantics,
    SemanticsResult, analyze_af, dung_framework, grounded_extension, search_complete_labelings,
    solve,
};
pub use export::{to_json, to_yaml};
pub use import::{Arena, ImportError, InvariantViolation, RefSource, from_json, from_yaml};
pub use metadata::{MetadataError, Value, parse_metadata};
pub use model::{
    Edge, Model, ModelArgument, ModelStatement, Node, PcsId, PcsIssue, RelationIssue, RelationKind,
    ResolvedPcs, ResolvedPcsItem, Role, build_model,
};
pub use sections::{Section, SectionId, Sections, build_sections};
pub use statements::{Statement, StatementConflict, StatementId, Statements, build_statements};
pub use tags::{TagId, Tags, build_tags};
