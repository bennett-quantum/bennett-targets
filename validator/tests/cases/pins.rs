use super::pins;

#[test]
fn pinned_sibling_never_falls_back_to_a_live_checkout() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let config = std::fs::read_to_string(root.join("gate.toml")).unwrap();
    let missing = config
        .lines()
        .map(|line| {
            if line.starts_with("sha = ") {
                format!("sha = \"{}\"", "0".repeat(40))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let error = pins::resolve(&root, &missing, "bennett-isa-spec").unwrap_err();
    assert!(error.contains("bennett-isa-spec@000000000000"), "{error}");
    assert!(error.contains("gate.toml"), "{error}");
    assert!(pins::isa().join("spec/Bennett-ISA-Spec-0.9.md").is_file());
}
