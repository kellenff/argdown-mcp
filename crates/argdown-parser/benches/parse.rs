//! Criterion benchmarks for `argdown_parser::parse`.
//!
//! Run the whole suite:
//!     cargo bench -p argdown-parser
//!
//! Regression-guard workflow (baselines live in the gitignored `target/criterion`).
//! Pass `--bench parse` so Criterion-only flags reach this bench and not the
//! crate's default libtest harness (which rejects them):
//!     cargo bench -p argdown-parser --bench parse -- --save-baseline main   # snapshot before a change
//!     cargo bench -p argdown-parser --bench parse -- --baseline main        # compare after
//!
//! A plain `cargo bench -p argdown-parser` (no forwarded flags) also auto-diffs
//! against the previous run and prints e.g. `change: +4.2% (p = 0.00)` — that is
//! the regression guard.
//!
//! For local HTML report plots, add Criterion's `html_reports` and `plotters`
//! features to the workspace dependency in the root `Cargo.toml` (we build with
//! `default-features = false` to keep the CI `--all-targets` compile lean, so the
//! plotting stack is not compiled by default).

use std::hint::black_box;

use argdown_parser::parse;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

// Feature micros: one small valid document per grammar construct, so a
// regression names the exact recognizer that slowed.
const HEADING: &str = "# A section heading at level one";
const STATEMENT: &str = "[Claim]: A plain claim of representative length.";
const RELATION: &str = "[A]\n  + [B]\n  - <C>\n    +> [D]\n  >< [E]";
const PCS: &str = "(1) first premise\n(2) second premise\n-- Modus Ponens --\n(3) conclusion";
const INLINE: &str = "Text with *italic*, **bold**, [a link](http://x.com), @[M], @<Arg>, #tag.";
const METADATA: &str = "[S]: a claim with data {certainty: 0.8, source: book}";
const FRONTMATTER: &str = "===\ntitle: Doc\nauthor: A\n===\n\n[S]: a claim";

const FEATURES: [(&str, &str); 7] = [
    ("heading", HEADING),
    ("statement", STATEMENT),
    ("relation", RELATION),
    ("pcs", PCS),
    ("inline", INLINE),
    ("metadata", METADATA),
    ("frontmatter", FRONTMATTER),
];

fn features(c: &mut Criterion) {
    let mut group = c.benchmark_group("features");
    for (name, src) in FEATURES {
        // A malformed corpus would make parse() fail-fast, so we'd time the
        // error path instead of parsing. Guard once, untimed.
        assert!(parse(src).is_ok(), "feature corpus {name:?} must parse");
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_function(name, |b| b.iter(|| parse(black_box(src))));
    }
    group.finish();
}

// Size scaling: one representative mixed "unit" repeated, so sizes are
// reproducible as code rather than committed fixture blobs.

/// Build a size-scaled corpus: document frontmatter once, then `units`
/// representative blocks (heading; a claim carrying inline markup + metadata; a
/// reference with a relation pair; a PCS). This unit is the intended
/// customization point — adjust it to match a realistic Argdown document.
fn corpus(units: usize) -> String {
    let mut s = String::from("===\ntitle: Benchmark Corpus\nauthor: bench\n===\n\n");
    for i in 0..units {
        s.push_str(&format!(
            "# Section {i}\n\n\
             [Claim {i}]: a claim with *italic*, **bold**, a [link](http://example.com/{i}), \
             @[Other {i}], @<Arg {i}> and a #tag {{certainty: 0.8}}\n\n\
             [Parent {i}]\n  + [Support {i}]\n  - <Counter {i}>\n\n\
             (1) first premise\n(2) second premise\n-- Modus Ponens --\n(3) conclusion {i}\n\n"
        ));
    }
    s
}

const SIZES: [(&str, usize); 3] = [("small", 1), ("medium", 50), ("large", 500)];

fn scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    for (name, units) in SIZES {
        let src = corpus(units);
        assert!(parse(&src).is_ok(), "scaling corpus {name:?} must parse");
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &src, |b, src| {
            b.iter(|| parse(black_box(src.as_str())))
        });
    }
    group.finish();
}

criterion_group!(benches, features, scaling);
criterion_main!(benches);
