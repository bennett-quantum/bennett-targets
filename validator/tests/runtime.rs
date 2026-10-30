use bennett_targets_validator::{
    expr::{self, Bindings},
    json, Target,
};
use serde_json::json;
fn target() -> Target {
    Target::load(include_str!(
        "../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json"
    ))
    .unwrap()
}
#[test]
fn matching_and_actual_bindings() {
    let t = target();
    for e in ["0", "0.001", "1"] {
        assert_eq!(
            t.native("cx", &["fast", "fast"], 1, 1, expr::decimal(e).unwrap())
                .unwrap()
                .unwrap()
                .cost["time"],
            expr::decimal("1").unwrap()
        );
    }
    assert!(t
        .native(
            "cx",
            &["fast", "fast", "fast"],
            1,
            2,
            expr::decimal("0.1").unwrap()
        )
        .unwrap()
        .is_none());
    let mut d = t.document.clone();
    d["gates"][0]["cost"]["time"] =
        json!({"op":"/","args":[1,{"op":"-","args":[{"var":"k"},{"var":"k"}]}]});
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert!(t
        .native("cx", &["fast"], 1, 1, expr::decimal("1").unwrap())
        .unwrap_err()
        .contains("division by zero"));
}
#[test]
fn scheme_expands_leaves_and_checks_indices() {
    let mut d = target().document;
    d["schemes"] = json!([{"name":"parity-expansion-probe","realises":{"op":"mcx","controls":{"min":1,"max":"unbounded"}},"ancillae":{"count":0,"state":"clean","class":"fast"},"template":[{"gate":"cx","wires":["c[i]","t"],"repeat":{"var":"i","from":0,"to":{"var":"k"}}}]}]);
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    let leaves = t
        .expand(
            "mcx",
            &["fast", "fast", "fast", "fast"],
            3,
            3,
            expr::decimal("0").unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(leaves.len(), 3);
    assert_eq!(
        leaves
            .iter()
            .map(|l| l.charge.cost["time"].clone())
            .sum::<expr::Rational>(),
        expr::decimal("3").unwrap()
    );
    // This is an accounting probe, not a correct MCX realisation: T3 witness.
    let controls = [true, false, false];
    let template = controls.iter().fold(false, |t, c| t ^ c);
    let reference = controls.iter().all(|x| *x);
    assert_ne!(template, reference);
    d["schemes"][0]["template"][0]["wires"][0] = json!("c[3]");
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert!(t
        .expand(
            "mcx",
            &["fast", "fast", "fast", "fast"],
            3,
            3,
            expr::decimal("0").unwrap(),
            None
        )
        .is_err());
}
#[test]
fn event_aggregation_and_capacity() {
    let t = target();
    let e = json::parse(include_str!(
        "../../../bennett-isa-spec/conformance/cases/E/E-aggregation/events.json"
    ))
    .unwrap();
    let mut d = t.document.clone();
    d["resources"]["gate_count"] = json!({"kind":"count","unit":"gate"});
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    let r = t.aggregate(&e).unwrap();
    assert_eq!(r["time"]["total"], expr::decimal("3").unwrap());
    assert_eq!(r["qubit_second"]["fast"], expr::decimal("4").unwrap());
    assert_eq!(r["peak_qubits"]["fast"], expr::decimal("2").unwrap());
    assert_eq!(r["gate_count"]["total"], expr::decimal("2").unwrap());
    let mut e = e.clone();
    e["leaves"][1]["start"] = json!(0);
    for i in 2..4 {
        e["live_intervals"][i]["start"] = json!(0);
        e["live_intervals"][i]["end"] = json!(1);
    }
    e["leaves"].as_array_mut().unwrap().pop();
    let r = t.aggregate(&e).unwrap();
    assert_eq!(r["time"]["total"], expr::decimal("1").unwrap());
    assert_eq!(r["peak_qubits"]["fast"], expr::decimal("4").unwrap());
    d["classes"]["fast"]["capacity"] = json!(3);
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert!(t.aggregate(&e).unwrap_err().contains("§8.4"));
}
#[test]
fn imported_quantity_conversion() {
    let mut d = target().document;
    d["resources"]["gates"] = json!({"kind":"count","unit":"gate"});
    d["imports"] = json!({"probe":{"version":"0.8","map":{"quantity":["gates",2]}}});
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert_eq!(
        t.import_quantities(
            "probe",
            &Bindings::from([("quantity".into(), expr::decimal("3").unwrap())])
        )
        .unwrap()["gates"],
        expr::decimal("6").unwrap()
    );
}
