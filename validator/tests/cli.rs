use std::{path::Path, process::Command};
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bennett-targets-validator")
}
#[test]
fn cli_canonicalizes_all_seven_fixtures_and_rejects_bad_validation() {
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
        if p.parent().unwrap().ends_with("E-topology-grid-09") {
            let case: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(p.parent().unwrap().join("case.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(
                value["target_hash"],
                case["expected_outcome"]["target_hash"]
            );
        }
        count += 1;
    }
    assert_eq!(count, 7);
    for name in [
        "E-mbu-must-be-false",
        "E-epsilon-zero-entry",
        "E-topology-grid-09",
    ] {
        let out = Command::new(binary())
            .arg("validate")
            .arg(root.join(name).join("target.json"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
        let message = error["error"].as_str().unwrap();
        assert!(message.contains('§'));
        if name == "E-topology-grid-09" {
            assert!(
                message.contains("§8.3") && message.contains("unknown field topology"),
                "{message}"
            );
        }
    }
    let out = Command::new(binary())
        .args(["validate", "/missing/file", "extra"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}
