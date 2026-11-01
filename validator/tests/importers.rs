use bennett_targets_validator::{importers, Target};
use serde_json::json;
fn base() -> Target {
    Target::load(include_str!(
        "../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json"
    ))
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
        "../../importers/fixtures/azure-qubit-params.json"
    ))
    .unwrap();
    assert!(importers::azure(&target, &source)
        .unwrap_err()
        .contains("unit"));
}

fn ideal() -> Target {
    Target::load(include_str!("../../targets/ideal-all-to-all.json")).unwrap()
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
fn azure_qec_formula_inputs_name_the_blocker() {
    let input = json!({"qubitParams":{"name":"p","instructionSet":"GateBased","oneQubitMeasurementTime":"1 us","oneQubitGateTime":"20 ns","oneQubitMeasurementErrorRate":0,"oneQubitGateErrorRate":0},"qecScheme":{"name":"surface_code"}});
    let err = importers::azure(&base(), &input).unwrap_err();
    assert!(
        err.contains("qecScheme") && err.contains("b-isa-spec-fd0"),
        "{err}"
    );
}
