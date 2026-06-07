//! Fuzz the full Layer-B pipeline on any parsed document. Layer B is **total**
//! (no `Result`): `build_model` / `build_tags` / `dung_framework` /
//! `grounded_extension` must never panic on any `Document` the parser produces.
//! Also checks build determinism and Dung framework/labelling well-formedness.
//!
//! Run: `cargo +nightly fuzz run model fuzz/seeds`

#![no_main]

use argdown_model::{build_model, build_tags, dung_framework, grounded_extension};
use argdown_parser::parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(doc) = parse(src) else {
        return;
    };

    let model = build_model(&doc);
    assert!(
        build_model(&doc) == model,
        "build_model is not deterministic"
    );
    let _tags = build_tags(&doc);
    let af = dung_framework(&model);
    let labelling = grounded_extension(&af);

    // Every attack endpoint is one of the framework's arguments.
    for &(from, to) in &af.attacks {
        assert!(
            af.arguments.contains(&from) && af.arguments.contains(&to),
            "attack endpoint is not an AF argument"
        );
    }
    // The grounded labelling partitions the arguments exactly once.
    assert_eq!(
        labelling.in_.len() + labelling.out.len() + labelling.undec.len(),
        af.arguments.len(),
        "grounded labelling does not partition the arguments"
    );
});
