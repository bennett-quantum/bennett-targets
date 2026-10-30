use bennett_targets_validator::validate;
#[test]
fn schema_and_canonical_red() {
    for bad in ["{}", "not json", "[]", "{\"x\":1,\"x\":2}"] {
        assert!(validate(bad).is_err(), "accepted {bad}");
    }
}
#[test]
fn suite_e_semantic_negatives_red() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bennett-isa-spec/conformance/cases/E");
    for name in ["E-epsilon-zero-entry", "E-mbu-must-be-false"] {
        let s = std::fs::read_to_string(base.join(name).join("target.json")).unwrap();
        assert!(validate(&s).is_err(), "accepted {name}");
    }
}
