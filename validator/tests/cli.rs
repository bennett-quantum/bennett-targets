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
    for name in ["E-mbu-must-be-false", "E-epsilon-zero-entry"] {
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
    }
    let out = Command::new(binary())
        .args(["validate", "/missing/file", "extra"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

fn query(command: &str, target: &Path, body: &str, name: &str) -> std::process::Output {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.tmp/cli-query");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}-{}.json", std::process::id()));
    std::fs::write(&path, body).unwrap();
    let out = Command::new(binary())
        .args([command.as_ref(), target.as_os_str(), path.as_os_str()])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    out
}

#[test]
fn cli_reports_the_published_topology_and_eps_decade_outcomes() {
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bennett-isa-spec/conformance/cases/E/E-topology-grid-09/target.json");
    let ok = query("adjacency", &target, r#"{"vertices":[0,1]}"#, "adjacent");
    assert_eq!(ok.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&ok.stdout).contains(r#""realisable":true"#));
    let no = query("adjacency", &target, r#"{"vertices":[0,3]}"#, "diagonal");
    assert_eq!(no.status.code(), Some(1));
    assert!(no.stdout.is_empty());
    let message = String::from_utf8_lossy(&no.stderr);
    assert!(message.contains("E_SCHEDULE_INFEASIBLE") && message.contains("§17 item 15"));
    let bad = query(
        "adjacency",
        &target,
        r#"{"vertices":[0,1],"x":1}"#,
        "unknown",
    );
    assert_eq!(bad.status.code(), Some(1));
    let bad = query(
        "eval",
        &target,
        r#"{"expr":{"table":"none","index":"eps_decade"},"bindings":{"eps":"0.1"}}"#,
        "table",
    );
    assert_eq!(bad.status.code(), Some(1));
    let value = query(
        "eval",
        &target,
        r#"{"expr":{"op":"+","args":[{"var":"eps"},1]},"bindings":{"eps":"0.1"}}"#,
        "value",
    );
    assert_eq!(value.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&value.stdout).contains("11/10"));
}
