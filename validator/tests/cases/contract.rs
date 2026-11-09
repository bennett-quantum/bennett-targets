use super::pins;
use bennett_targets_validator::validate;
#[test]
fn schema_and_canonical_red() {
    for bad in ["{}", "not json", "[]", "{\"x\":1,\"x\":2}"] {
        assert!(validate(bad).is_err(), "accepted {bad}");
    }
}
#[test]
fn suite_e_semantic_negatives_red() {
    let base = pins::isa().join("conformance/cases/E");
    for name in ["E-epsilon-zero-entry", "E-mbu-must-be-false"] {
        let s = std::fs::read_to_string(base.join(name).join("target.json")).unwrap();
        assert!(validate(&s).is_err(), "accepted {name}");
    }
}

#[test]
fn pointer_width_list_is_a_capability_not_a_fixed_enum() {
    let mut d = bennett_targets_validator::json::parse(
        &std::fs::read_to_string(
            pins::isa().join("conformance/cases/E/E-jcs-target-hash/target.json"),
        )
        .unwrap(),
    )
    .unwrap();
    d["ptr_widths_supported"] = serde_json::json!([16, 32, 128]);
    assert!(validate(&bennett_targets_validator::json::canonical(&d).unwrap()).is_ok());
}

#[test]
fn optional_params_is_an_array_when_present() {
    let mut d = bennett_targets_validator::json::parse(
        &std::fs::read_to_string(
            pins::isa().join("conformance/cases/E/E-jcs-target-hash/target.json"),
        )
        .unwrap(),
    )
    .unwrap();
    d["gates"][0]["params"] = serde_json::Value::Null;
    assert!(validate(&bennett_targets_validator::json::canonical(&d).unwrap()).is_err());
}

#[test]
fn schema_rejects_invalid_class_and_capability_fields() {
    use bennett_targets_validator::{json, Target};
    use serde_json::json;
    let base = json::parse(include_str!("../../../targets/ideal-quantum-circuit.json")).unwrap();
    for (pointer, value, expected) in [
        ("/classes", json!({}), "classes must not be empty"),
        (
            "/ptr_widths_supported",
            json!([]),
            "ptr_widths_supported must not be empty",
        ),
        (
            "/ptr_widths_supported",
            json!([32, 32]),
            "unique positive pointer widths",
        ),
        (
            "/capabilities/measurement/mid_circuit",
            json!(null),
            "expected bool",
        ),
        (
            "/gates/0/classes",
            json!([]),
            "class pattern cannot be empty",
        ),
        ("/objective", json!({"unknown":[]}), "unknown objective"),
    ] {
        let mut document = base.clone();
        *document.pointer_mut(pointer).unwrap() = value;
        let error = Target::load(&json::canonical(&document).unwrap()).unwrap_err();
        assert!(error.contains(expected), "{pointer}: {error}");
    }
}
