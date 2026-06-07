//! Criterion benchmarks for the Layer-B model builders (`argdown_model`).
//!
//! Run the whole suite:
//!     cargo bench -p argdown-model
//!
//! Regression-guard workflow (baselines live in the gitignored `target/criterion`).
//! Pass `--bench model` so Criterion-only flags reach this bench and not the
//! crate's default libtest harness (which rejects them):
//!     cargo bench -p argdown-model --bench model -- --save-baseline main   # snapshot before a change
//!     cargo bench -p argdown-model --bench model -- --baseline main        # compare after
//!
//! A plain `cargo bench -p argdown-model` also auto-diffs against the previous
//! run and prints e.g. `change: +4.2% (p = 0.00)` — that is the regression guard.
//!
//! Layer-B functions take `&Document` / `&Model` / `&ArgumentationFramework`, so
//! each bench parses/builds its input **once, untimed** (parse cost is the
//! parser bench's concern) and times only the Layer-B call.

use std::hint::black_box;

use argdown_core::{Block, Document, Metadata};
use argdown_model::{
    build_arguments, build_model, build_sections, build_statements, build_tags, dung_framework,
    grounded_extension, parse_metadata,
};
use argdown_parser::parse;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

/// Mixed corpus: frontmatter once, then `units` representative blocks (heading;
/// a claim with inline markup + metadata; an argument; a reference with a
/// relation pair; a PCS). Reproducible as code rather than a committed fixture;
/// the intended customization point for a realistic document shape.
fn corpus(units: usize) -> String {
    let mut s = String::from("===\ntitle: Benchmark Corpus\nauthor: bench\n===\n\n");
    for i in 0..units {
        s.push_str(&format!(
            "# Section {i}\n\n\
             [Claim {i}]: a claim with *italic*, **bold**, a [link](http://example.com/{i}), \
             @[Other {i}], @<Arg {i}> and a #tag {{certainty: 0.8}}\n\n\
             <Argument {i}>: a defended claim supported by reasons.\n\n\
             [Parent {i}]\n  + [Support {i}]\n  - <Counter {i}>\n\n\
             (1) first premise\n(2) second premise\n-- Modus Ponens --\n(3) conclusion {i}\n\n"
        ));
    }
    s
}

/// Dialectic corpus: `args` arguments in an argument-level attack chain
/// (`<Arg i>` is attacked by `<Arg i+1>`), so the Dung framework is non-trivial
/// (`args - 1` attacks). The Dung benches scale with the attack graph, not bytes.
fn dialectic(args: usize) -> String {
    let mut s = String::new();
    for i in 0..args {
        s.push_str(&format!("<Arg {i}>: argument number {i}\n"));
        if i + 1 < args {
            s.push_str(&format!("  - <Arg {}>\n", i + 1));
        }
        s.push('\n');
    }
    s
}

/// Pull a `Metadata` block out of a leading statement, to bench `parse_metadata`.
fn extract_metadata(doc: &Document) -> Metadata {
    match doc.blocks.first() {
        Some(Block::Statement(s)) => s.metadata.clone().expect("statement carries metadata"),
        _ => panic!("expected a leading statement with metadata"),
    }
}

const SIZES: [(&str, usize); 3] = [("small", 1), ("medium", 50), ("large", 500)];
const DUNG_SIZES: [(&str, usize); 3] = [("small", 10), ("medium", 100), ("large", 500)];

// One micro per public Layer-B function, over a fixed representative input, so a
// regression names the exact slice. Inputs are built once, outside the loop.
fn functions(c: &mut Criterion) {
    let mixed_src = corpus(20);
    let mixed_doc = parse(&mixed_src).expect("mixed corpus parses");

    let dia_doc = parse(&dialectic(30)).expect("dialectic corpus parses");
    let dia_model = build_model(&dia_doc);
    let dia_af = dung_framework(&dia_model);
    assert!(!dia_af.attacks.is_empty(), "dialectic AF must have attacks");

    let meta_doc = parse("[S]: claim {certainty: 0.8, source: a book, tags: [alpha, beta, gamma]}")
        .expect("metadata corpus parses");
    let meta = extract_metadata(&meta_doc);

    let mut group = c.benchmark_group("functions");
    group.bench_function("sections", |b| {
        b.iter(|| build_sections(black_box(&mixed_doc)))
    });
    group.bench_function("statements", |b| {
        b.iter(|| build_statements(black_box(&mixed_doc)))
    });
    group.bench_function("arguments", |b| {
        b.iter(|| build_arguments(black_box(&mixed_doc)))
    });
    group.bench_function("model", |b| b.iter(|| build_model(black_box(&mixed_doc))));
    group.bench_function("tags", |b| b.iter(|| build_tags(black_box(&mixed_doc))));
    group.bench_function("parse_metadata", |b| {
        b.iter(|| parse_metadata(black_box(&meta)))
    });
    group.bench_function("dung_framework", |b| {
        b.iter(|| dung_framework(black_box(&dia_model)))
    });
    group.bench_function("grounded_extension", |b| {
        b.iter(|| grounded_extension(black_box(&dia_af)))
    });
    group.finish();
}

// End-to-end Layer-B cost: `build_model` (the aggregate) over the mixed corpus
// at increasing sizes, reported as MB/s.
fn scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    for (name, units) in SIZES {
        let src = corpus(units);
        let doc = parse(&src).expect("scaling corpus parses");
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &doc, |b, doc| {
            b.iter(|| build_model(black_box(doc)))
        });
    }
    group.finish();
}

// Dung scaling: the framework projection and the grounded-labelling fixpoint
// over attack chains of increasing argument count (elements/s).
fn dung_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("dung_scaling");
    for (name, args) in DUNG_SIZES {
        let doc = parse(&dialectic(args)).expect("dialectic corpus parses");
        let model = build_model(&doc);
        let af = dung_framework(&model);
        assert!(!af.attacks.is_empty(), "dialectic AF must have attacks");
        group.throughput(Throughput::Elements(args as u64));
        group.bench_with_input(BenchmarkId::new("framework", name), &model, |b, model| {
            b.iter(|| dung_framework(black_box(model)))
        });
        group.bench_with_input(BenchmarkId::new("grounded", name), &af, |b, af| {
            b.iter(|| grounded_extension(black_box(af)))
        });
    }
    group.finish();
}

criterion_group!(benches, functions, scaling, dung_scaling);
criterion_main!(benches);
