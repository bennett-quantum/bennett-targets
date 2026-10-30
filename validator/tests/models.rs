use bennett_targets_validator::Target;
#[test]
fn models_are_canonical_and_have_provenance() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../targets");
    for name in [
        "ideal-all-to-all",
        "superconducting-grid-illustrative",
        "trapped-ion-illustrative",
    ] {
        let path = root.join(format!("{name}.json"));
        let s = std::fs::read_to_string(&path).expect("model must exist");
        Target::load(&s).unwrap();
        let p = std::fs::read_to_string(root.join(format!("{name}.provenance.md"))).unwrap();
        assert!(p.contains("illustrative"));
    }
}
