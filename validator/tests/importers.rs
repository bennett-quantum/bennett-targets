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
    assert_eq!(report["mapping"].as_object().unwrap().len(), 10);
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
        let mut bad = input.clone();
        bad["qubitParams"][key] = json!(0.001);
        assert!(importers::azure(&base(), &bad).unwrap_err().contains(key));
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
