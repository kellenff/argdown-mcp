//! Fuzz `argdown_parser::parse`: it must never panic, must be deterministic,
//! and every span it emits must index within the source ("every span is a
//! source range").
//!
//! Run: `cargo +nightly fuzz run parse fuzz/seeds`

#![no_main]

use argdown_core::{Argument, Block, Document, Metadata, PcsItem, RelationTarget, Span, Statement};
use argdown_parser::parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The parser's input is text; feed it valid UTF-8 and skip the rest.
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(doc) = parse(src) else {
        return;
    };
    // Parsing is deterministic: the same input yields the same AST.
    assert!(
        parse(src).as_ref() == Ok(&doc),
        "parse is not deterministic"
    );
    // Every span indexes within the source.
    check_document(&doc, src.len());
});

fn check_span(span: Span, len: usize) {
    assert!(
        span.start <= span.end && span.end <= len,
        "span {span:?} out of bounds for source length {len}"
    );
}

fn check_meta(meta: &Option<Metadata>, len: usize) {
    if let Some(m) = meta {
        check_span(m.span, len);
    }
}

fn check_statement(s: &Statement, len: usize) {
    check_span(s.span, len);
    check_meta(&s.metadata, len);
    for inline in &s.inlines {
        check_span(inline.span, len);
    }
}

fn check_argument(a: &Argument, len: usize) {
    check_span(a.span, len);
    check_meta(&a.metadata, len);
    for inline in &a.inlines {
        check_span(inline.span, len);
    }
}

fn check_target(target: &RelationTarget, len: usize) {
    match target {
        RelationTarget::Statement(s) => check_statement(s, len),
        RelationTarget::Argument(a) => check_argument(a, len),
    }
}

fn check_document(doc: &Document, len: usize) {
    check_meta(&doc.frontmatter, len);
    for block in &doc.blocks {
        match block {
            Block::Heading(h) => {
                check_span(h.span, len);
                check_meta(&h.metadata, len);
            }
            Block::Statement(s) => check_statement(s, len),
            Block::Argument(a) => check_argument(a, len),
            Block::Relation(r) => {
                check_span(r.span, len);
                check_target(&r.target, len);
            }
            Block::Pcs(p) => {
                check_span(p.span, len);
                for item in &p.items {
                    match item {
                        PcsItem::Statement {
                            statement, span, ..
                        } => {
                            check_span(*span, len);
                            check_statement(statement, len);
                        }
                        PcsItem::Inference { metadata, span, .. } => {
                            check_span(*span, len);
                            check_meta(metadata, len);
                        }
                        PcsItem::Relation(r) => {
                            check_span(r.span, len);
                            check_target(&r.target, len);
                        }
                    }
                }
            }
        }
    }
}
