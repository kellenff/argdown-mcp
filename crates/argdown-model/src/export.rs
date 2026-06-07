//! Serialize the Layer-B [`Model`] to JSON and YAML.
//!
//! A plain serde dump: the wire shape mirrors the Rust types as-is (stable
//! source-order ids and byte spans included), with serde's default
//! externally-tagged enum representation. The validating import counterpart is
//! [`crate::from_json`]/[`crate::from_yaml`]. Metadata values (`Value`) come
//! from `noyalib`'s `serde_yaml` and (de)serialize through both backends.

use crate::Model;

/// Serialize a [`Model`] to pretty-printed JSON.
///
/// Returns `Err` only if serialization itself fails — in practice that means
/// parsed metadata (`Value`) held a non-string mapping key (e.g. `{1: x}`)
/// that JSON cannot represent. The error is surfaced as data rather than
/// panicking, matching the crate's total style.
pub fn to_json(model: &Model) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(model)
}

/// Serialize a [`Model`] to YAML (via `noyalib`'s `serde_yaml`).
pub fn to_yaml(model: &Model) -> Result<String, noyalib::compat::serde_yaml::Error> {
    noyalib::compat::serde_yaml::to_string(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_model;
    use argdown_parser::parse;
    use serde_json::Value as Json;

    /// A small but representative document: a named argument owning a one-step
    /// PCS, plus a top-level support relation that yields an edge.
    const SAMPLE: &str = "<A>: d\n\n(1) P1\n----\n(2) C1\n\n[X]: x\n  + [Y]: y";

    #[test]
    fn json_has_the_model_top_level_keys() {
        let model = build_model(&parse(SAMPLE).unwrap());
        let json = to_json(&model).expect("model serializes to JSON");
        let value: Json = serde_json::from_str(&json).expect("emitted JSON reparses");

        let obj = value.as_object().expect("top level is a JSON object");
        for key in ["statements", "arguments", "pcs", "edges", "block_pcs"] {
            assert!(obj.contains_key(key), "missing top-level key {key:?}");
        }
    }

    #[test]
    fn json_preserves_pcs_item_count() {
        let model = build_model(&parse(SAMPLE).unwrap());
        let json = to_json(&model).unwrap();
        let value: Json = serde_json::from_str(&json).unwrap();

        let items = value["pcs"][0]["items"]
            .as_array()
            .expect("pcs[0].items is an array");
        assert_eq!(items.len(), model.pcs[0].items.len());
        // 0=premise 1=inference 2=main-conclusion.
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn yaml_round_trips_through_serde_yaml() {
        let model = build_model(&parse(SAMPLE).unwrap());
        let yaml = to_yaml(&model).expect("model serializes to YAML");

        let value: noyalib::compat::serde_yaml::Value =
            noyalib::compat::serde_yaml::from_str(&yaml).expect("emitted YAML reparses");
        let statements = value["statements"]
            .as_sequence()
            .expect("statements is a YAML sequence");
        assert_eq!(statements.len(), model.statements.len());
    }

    #[test]
    fn empty_model_serializes_to_stable_shapes() {
        let model = Model::default();

        let json: Json = serde_json::from_str(&to_json(&model).unwrap()).unwrap();
        assert_eq!(json["statements"], serde_json::json!([]));
        assert_eq!(json["edges"], serde_json::json!([]));

        let yaml: noyalib::compat::serde_yaml::Value =
            noyalib::compat::serde_yaml::from_str(&to_yaml(&model).unwrap()).unwrap();
        assert!(yaml["statements"].as_sequence().unwrap().is_empty());
    }
}
