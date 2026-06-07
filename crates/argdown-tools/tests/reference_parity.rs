//! Reference cross-check for the B6b canonical grounded probe.
//!
//! The project's "track the reference" convention (see
//! `docs/snowball/plans/2026-06-06-argdown-mcp-server.md`, Step 5) expects
//! `dung_extensions` on `<A>: a\n\n<B>: b\n  -> <A>` to yield `{B in, A out}`.
//! Live `@argdown/core` MCP was unavailable during v1.0 verification (npm
//! `@argdown/core@2.0.1` exposes no Dung API; `jsr:@argdown/cli` fetch failed).
//! This test encodes the B6b probe so regressions are caught in CI.

use argdown_model::Semantics;
use argdown_tools::{dung, extensions};

const B6B_CANONICAL: &str = "<A>: a\n\n<B>: b\n  -> <A>";

fn titles(refs: &[argdown_tools::ArgRef]) -> Vec<String> {
    refs.iter()
        .filter_map(|a| a.title.clone())
        .collect()
}

#[test]
fn b6b_grounded_partition_is_b_in_a_out() {
    let d = dung(B6B_CANONICAL).expect("valid source");
    assert_eq!(titles(&d.in_), vec!["B"]);
    assert_eq!(titles(&d.out), vec!["A"]);
    assert!(d.undec.is_empty(), "expected empty undec, got {:?}", d.undec);
}

#[test]
fn extensions_grounded_matches_dung_alias_on_b6b_probe() {
    let legacy = dung(B6B_CANONICAL).unwrap();
    let modern = extensions(B6B_CANONICAL, Semantics::Grounded).unwrap();
    assert_eq!(modern.labellings.len(), 1);
    let in_ids: Vec<_> = modern.extension_sets[0].iter().map(|a| a.id).collect();
    let legacy_in: Vec<_> = legacy.in_.iter().map(|a| a.id).collect();
    assert_eq!(in_ids, legacy_in);
}
