use bennett_targets_validator::validate;
#[test]
fn schema_and_canonical_red() {
    for bad in ["{}", "not json", "[]", "{\"x\":1,\"x\":2}"] {
        assert!(validate(bad).is_err(), "accepted {bad}");
    }
}
#[test]
fn suite_e_semantic_negatives_red() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bennett-isa-spec/conformance/cases/E");
    for name in ["E-epsilon-zero-entry", "E-mbu-must-be-false"] {
        let s = std::fs::read_to_string(base.join(name).join("target.json")).unwrap();
        assert!(validate(&s).is_err(), "accepted {name}");
    }
}

#[test]
fn pointer_width_list_is_a_capability_not_a_fixed_enum() {
    let mut d = bennett_targets_validator::json::parse(include_str!(
        "../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json"
    ))
    .unwrap();
    d["ptr_widths_supported"] = serde_json::json!([16, 32, 128]);
    assert!(validate(&bennett_targets_validator::json::canonical(&d).unwrap()).is_ok());
}

#[test]
fn optional_params_is_an_array_when_present() {
    let mut d = bennett_targets_validator::json::parse(include_str!(
        "../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json"
    ))
    .unwrap();
    d["gates"][0]["params"] = serde_json::Value::Null;
    assert!(validate(&bennett_targets_validator::json::canonical(&d).unwrap()).is_err());
}
