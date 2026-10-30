use std::{path::Path, process::Command};
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bennett-targets-validator")
}
#[test]
fn cli_canonicalizes_all_six_fixtures_and_rejects_bad_validation() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bennett-isa-spec/conformance/cases/E");
    let mut count = 0;
    for d in std::fs::read_dir(&root).unwrap() {
        let p = d.unwrap().path().join("target.json");
        if !p.exists() {
            continue;
        }
        let out = Command::new(binary())
            .arg("canon")
            .arg(&p)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            value["canonical"].as_str().unwrap().as_bytes(),
            std::fs::read(&p).unwrap()
        );
        count += 1;
    }
    assert_eq!(count, 6);
    for name in ["E-mbu-must-be-false", "E-epsilon-zero-entry"] {
        let out = Command::new(binary())
            .arg("validate")
            .arg(root.join(name).join("target.json"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
        assert!(error["error"].as_str().unwrap().contains('§'));
    }
    let out = Command::new(binary())
        .args(["validate", "/missing/file", "extra"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}
