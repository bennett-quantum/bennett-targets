use super::pins;
use bennett_targets_validator::{importers, Target};
use serde_json::json;
fn base() -> Target {
    Target::load(
        &std::fs::read_to_string(
            pins::isa().join("conformance/cases/E/E-jcs-target-hash/target.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn qualtran_all_fields_and_unknowns() {
    let input =
        json!({"t":1,"toffoli":2,"cswap":3,"and_bloq":4,"clifford":5,"rotation":6,"measurement":7});
    let report = importers::qualtran(&base(), &input).unwrap();
    assert_eq!(report["mapping"].as_object().unwrap().len(), 7);
    let t = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    assert_eq!(
        t.document["imports"]["qualtran.GateCounts"]["map"]["cswap"],
        json!(["qualtran.cswap", 1])
    );
    assert_eq!(report["quantities"]["qualtran.and_bloq"], json!(4));
    for field in input.as_object().unwrap().keys() {
        assert_eq!(report["mapping"][field]["defaulted"], false, "{field}");
        assert_eq!(
            report["quantities"][format!("qualtran.{field}")],
            input[field]
        );
    }
    for bad in [
        json!({"unknown":1}),
        json!({"t":"n+1"}),
        json!({"t":-1}),
        json!({"t":1.5}),
    ] {
        assert!(importers::qualtran(&base(), &bad).is_err());
    }
}
#[test]
fn azure_times_and_errors_are_never_silent() {
    let input = json!({"qubitParams":{"name":"illustrative","instructionSet":"GateBased","oneQubitMeasurementTime":"1 us","oneQubitGateTime":"20 ns","twoQubitGateTime":"200 ns","tGateTime":"30 ns","oneQubitMeasurementErrorRate":0,"oneQubitGateErrorRate":0,"twoQubitGateErrorRate":0,"tGateErrorRate":0}});
    let report = importers::azure(&base(), &input).unwrap();
    assert_eq!(report["mapping"].as_object().unwrap().len(), 11);
    let t = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    assert!(t.document["gates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["op"] == "cx" && g["cost"]["time"] == json!(0.0000002)));
    for key in [
        "oneQubitMeasurementErrorRate",
        "oneQubitGateErrorRate",
        "twoQubitGateErrorRate",
        "tGateErrorRate",
    ] {
        for out_of_range in [json!(1.5), json!(-0.1), json!("0.1")] {
            let mut bad = input.clone();
            bad["qubitParams"][key] = out_of_range;
            assert!(importers::azure(&base(), &bad).unwrap_err().contains(key));
        }
    }
    for key in [
        "errorBudget",
        "qecScheme",
        "constraints",
        "distillationUnitSpecifications",
        "profiling",
        "estimateType",
        "resumeAfterFailedIteration",
    ] {
        let mut bad = input.clone();
        bad[key] = json!({});
        assert!(importers::azure(&base(), &bad).unwrap_err().contains(key));
    }
    let mut bad = input.clone();
    bad["qubitParams"]["oneQubitGateTime"] = json!("20 fortnight");
    assert!(importers::azure(&base(), &bad).is_err());
}

#[test]
fn importer_rejects_unit_mismatch_and_preserves_huge_counts() {
    let input =
        bennett_targets_validator::json::parse(r#"{"t":123456789012345678901234567890}"#).unwrap();
    let out = importers::qualtran(&base(), &input).unwrap();
    assert_eq!(
        out["quantities"]["qualtran.t"],
        json!("123456789012345678901234567890")
    );
    let mut target = base().document;
    target["resources"]["time"]["unit"] = json!("ns");
    let target =
        Target::load(&bennett_targets_validator::json::canonical(&target).unwrap()).unwrap();
    let source = bennett_targets_validator::json::parse(include_str!(
        "../../../importers/fixtures/azure-qubit-params.json"
    ))
    .unwrap();
    assert_eq!(
        importers::azure(&target, &source).unwrap_err(),
        "import: base time unit must be second or s"
    );
}

fn ideal() -> Target {
    Target::load(include_str!("../../../targets/ideal-all-to-all.json")).unwrap()
}

/// North Star §7: not-aligned quantities import as new named resources. A
/// failure probability becomes a §8.2 count resource and never touches epsilon.
#[test]
fn azure_nonzero_rates_are_named_resources_never_epsilon() {
    let input = json!({"qubitParams":{"name":"p","instructionSet":"GateBased","oneQubitMeasurementTime":"1 us","oneQubitGateTime":"20 ns","oneQubitMeasurementErrorRate":{"process":0.002,"readout":0.003},"oneQubitGateErrorRate":0.001,"twoQubitGateErrorRate":0.01,"idleErrorRate":0.0001}});
    let before = ideal();
    let report = importers::azure(&before, &input).unwrap();
    let t = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    let gates = t.document["gates"].as_array().unwrap();
    let old = before.document["gates"].as_array().unwrap();
    assert_eq!(gates.len(), old.len());
    for (new, old) in gates.iter().zip(old) {
        assert_eq!(new["epsilon"], old["epsilon"], "{}", new["op"]);
    }
    let charged = |op: &str, r: &str| -> serde_json::Value {
        gates.iter().find(|g| g["op"] == op).unwrap()["cost"][r].clone()
    };
    assert_eq!(charged("x", "azure.oneQubitGateErrorRate"), json!(0.001));
    assert_eq!(charged("cx", "azure.twoQubitGateErrorRate"), json!(0.01));
    // tGateErrorRate defaults to the one-qubit rate; the report says so.
    assert_eq!(charged("t", "azure.tGateErrorRate"), json!(0.001));
    assert_eq!(
        report["mapping"]["tGateErrorRate"]["defaulted"],
        json!(true)
    );
    let q = &report["quantities"];
    assert_eq!(
        q["azure.oneQubitMeasurementErrorRate.process"],
        json!(0.002)
    );
    assert_eq!(
        q["azure.oneQubitMeasurementErrorRate.readout"],
        json!(0.003)
    );
    assert_eq!(q["azure.idleErrorRate"], json!(0.0001));
    for r in ["azure.oneQubitGateErrorRate", "azure.idleErrorRate"] {
        assert_eq!(t.document["resources"][r]["kind"], "count");
    }
    // No gate is charged for a measurement or idle quantity: §8 has no such event.
    assert!(gates
        .iter()
        .all(|g| g["cost"].get("azure.idleErrorRate").is_none()));
}

#[test]
fn azure_majorana_quantities_import_without_inventing_gates() {
    let input = json!({"qubitParams":{"name":"m","instructionSet":"Majorana","oneQubitMeasurementTime":"100 ns","twoQubitJointMeasurementTime":"200 ns","tGateTime":"1 us","oneQubitMeasurementErrorRate":{"process":0.0001,"readout":0.0001},"tGateErrorRate":0.05}});
    let before = ideal();
    let report = importers::azure(&before, &input).unwrap();
    let t = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    assert_eq!(
        t.document["gates"].as_array().unwrap().len(),
        before.document["gates"].as_array().unwrap().len()
    );
    let q = &report["quantities"];
    assert_eq!(
        bennett_targets_validator::expr::number(&q["azure.twoQubitJointMeasurementTime.seconds"])
            .unwrap(),
        bennett_targets_validator::expr::decimal("0.0000002").unwrap()
    );
    assert_eq!(
        q["azure.twoQubitJointMeasurementErrorRate.readout"],
        json!(0.0001)
    );
    let cx = t.document["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["op"] == "cx")
        .unwrap();
    let old = before.document["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["op"] == "cx")
        .unwrap();
    assert_eq!(cx["cost"]["time"], old["cost"]["time"]);
    let mut bad = input.clone();
    bad["qubitParams"]["oneQubitGateTime"] = json!("1 ns");
    assert!(importers::azure(&before, &bad)
        .unwrap_err()
        .contains("oneQubitGateTime"));
    let mut bad = input;
    bad["qubitParams"]["instructionSet"] = json!("Other");
    assert!(importers::azure(&before, &bad)
        .unwrap_err()
        .contains("instructionSet"));
}

#[test]
fn azure_qec_formula_inputs_name_the_missing_contract() {
    let input = json!({"qubitParams":{"name":"p","instructionSet":"GateBased","oneQubitMeasurementTime":"1 us","oneQubitGateTime":"20 ns","oneQubitMeasurementErrorRate":0,"oneQubitGateErrorRate":0},"qecScheme":{"name":"surface_code"}});
    let err = importers::azure(&base(), &input).unwrap_err();
    assert!(
        err.contains("qecScheme") && err.contains("ISA §8.3") && err.contains("QEC"),
        "{err}"
    );
}

#[test]
fn real_qualtran_qft_dump_preserves_counts_and_marks_each_zero_default() {
    use bennett_targets_validator::{expr, json};
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../importers/fixtures/qualtran-qft8-real.json"
    ))
    .unwrap();
    let input = json::parse(&source).unwrap();
    // Produced by Qualtran 0.7.0 QFTTextBook(8), not hand-entered GateCounts.
    assert_eq!(
        input,
        json!({"t":6,"and_bloq":28,"clifford":47,"rotation":15,"measurement":28})
    );
    let report = importers::qualtran(&ideal(), &input).unwrap();
    let target = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    for (field, expected) in [
        ("t", 6),
        ("toffoli", 0),
        ("cswap", 0),
        ("and_bloq", 28),
        ("clifford", 47),
        ("rotation", 15),
        ("measurement", 28),
    ] {
        assert_eq!(
            report["mapping"][field]["defaulted"],
            input.get(field).is_none(),
            "{field}"
        );
        assert_eq!(
            report["quantities"][format!("qualtran.{field}")],
            expected,
            "{field}"
        );
        assert_eq!(
            target.document["imports"]["qualtran.GateCounts"]["map"][field],
            json!([format!("qualtran.{field}"), 1])
        );
    }
    let quantities = input
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), expr::number(v).unwrap()))
        .collect();
    let converted = target
        .import_quantities("qualtran.GateCounts", &quantities)
        .unwrap();
    assert_eq!(converted["qualtran.rotation"], expr::decimal("15").unwrap());
}

#[test]
fn real_azure_estimator_report_has_an_explicit_hardware_projection() {
    use bennett_targets_validator::{expr, json};
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../importers/fixtures");
    let read = |name| json::parse(&std::fs::read_to_string(fixtures.join(name)).unwrap()).unwrap();
    let full = read("azure-estimator-report-real.json");
    let input = read("azure-estimator-qubit-params-real.json");
    assert_eq!(input["qubitParams"], full["jobParams"]["qubitParams"]);
    let error = importers::azure(&ideal(), &full).unwrap_err();
    assert!(
        error.contains("errorBudget") && error.contains("unsupported field"),
        "{error}"
    );
    let before = ideal();
    let report = importers::azure(&before, &input).unwrap();
    let target = Target::load(report["canonical"].as_str().unwrap()).unwrap();
    assert_eq!(report["source"]["name"], "qubit_gate_ns_e3");
    for (op, classes, k) in [
        ("x", vec!["data"], 0),
        ("cx", vec!["data", "data"], 1),
        ("t", vec!["data"], 0),
    ] {
        let charge = target
            .native(op, &classes, k, k, expr::decimal("0").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(
            charge.cost["time"],
            expr::decimal("0.00000005").unwrap(),
            "{op}"
        );
        assert_eq!(charge.epsilon, expr::decimal("0").unwrap());
    }
    for field in input["qubitParams"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| !["name", "instructionSet"].contains(&key.as_str()))
    {
        assert_eq!(report["mapping"][field]["defaulted"], false, "{field}");
    }
    assert_eq!(
        expr::number(&report["quantities"]["azure.oneQubitMeasurementTime.seconds"]).unwrap(),
        expr::decimal("0.0000001").unwrap()
    );
    assert_eq!(
        report["quantities"]["azure.oneQubitGateErrorRate"],
        json!(0.001)
    );
    for (new, old) in target.document["gates"]
        .as_array()
        .unwrap()
        .iter()
        .zip(before.document["gates"].as_array().unwrap())
    {
        assert_eq!(new["epsilon"], old["epsilon"]);
    }
}

#[test]
fn real_sample_sources_have_dates_urls_and_byte_hashes() {
    use bennett_targets_validator::json;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../importers/fixtures");
    for name in [
        "qualtran-qft8-real",
        "azure-estimator-report-real",
        "azure-estimator-qubit-params-real",
    ] {
        let metadata = json::parse(
            &std::fs::read_to_string(root.join(format!("{name}.provenance.json"))).unwrap(),
        )
        .unwrap();
        assert!(metadata["source_url"]
            .as_str()
            .unwrap()
            .starts_with("https://"));
        let date = metadata["retrieved_utc"].as_str().unwrap();
        assert_eq!(date.len(), 20);
        assert!(date.ends_with('Z'));
        assert_eq!(
            metadata["sha256"],
            json::hash(&std::fs::read(root.join(format!("{name}.json"))).unwrap())
        );
    }
}
