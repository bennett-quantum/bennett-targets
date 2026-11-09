use super::pins;
use std::{path::Path, process::Command};
fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bennett-targets-validator")
}
#[test]
fn cli_canonicalizes_all_seven_fixtures_and_rejects_bad_validation() {
    let root = pins::isa().join("conformance/cases/E");
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
    let target = pins::isa().join("conformance/cases/E/E-topology-grid-09/target.json");
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

#[test]
fn cli_queries_reject_malformed_fields_with_json_diagnostics() {
    let target = pins::isa().join("conformance/cases/E/E-topology-grid-09/target.json");
    for (command, body, error) in [
        ("eval", "null", "query: expected object"),
        ("eval", r#"{"expr":0}"#, "query: missing bindings"),
        (
            "eval",
            r#"{"expr":0,"bindings":null}"#,
            "query: bindings object",
        ),
        (
            "eval",
            r#"{"expr":0,"bindings":{"eps":0.1}}"#,
            "query: binding values are decimal strings",
        ),
        ("adjacency", r#"{"vertices":0}"#, "query: vertices array"),
    ] {
        let out = query(command, &target, body, "invalid-query");
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
        assert_eq!(diagnostic, serde_json::json!({"error":error}));
    }
    for args in [
        vec![],
        vec!["eval"],
        vec!["eval", target.to_str().unwrap()],
        vec!["import-azure", target.to_str().unwrap()],
        vec!["unknown", target.to_str().unwrap()],
    ] {
        let out = Command::new(binary()).args(&args).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(out.stdout.is_empty());
        let diagnostic: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
        assert!(!diagnostic["error"].as_str().unwrap().is_empty());
    }
}

#[test]
fn cli_imports_real_samples_into_loadable_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (command, filename, resource, quantity) in [
        (
            "import-qualtran",
            "qualtran-qft8-real.json",
            "qualtran.and_bloq",
            serde_json::json!(28),
        ),
        (
            "import-azure",
            "azure-estimator-qubit-params-real.json",
            "azure.oneQubitGateErrorRate",
            serde_json::json!(0.001),
        ),
    ] {
        let output = Command::new(binary())
            .arg(command)
            .arg(root.join("targets/ideal-quantum-circuit.json"))
            .arg(root.join("importers/fixtures").join(filename))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let target =
            bennett_targets_validator::Target::load(report["canonical"].as_str().unwrap()).unwrap();
        assert_eq!(report["target_hash"], target.hash);
        assert_eq!(report["quantities"][resource], quantity);
    }
}
