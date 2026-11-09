use super::pins;
use bennett_targets_validator::{
    expr::{self, Bindings},
    json, Target,
};
use serde_json::json;
fn target() -> Target {
    Target::load(
        &std::fs::read_to_string(
            pins::isa().join("conformance/cases/E/E-jcs-target-hash/target.json"),
        )
        .unwrap(),
    )
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
        .native("cx", &["fast", "fast"], 1, 1, expr::decimal("1").unwrap())
        .unwrap_err()
        .contains("division by zero"));
}

#[test]
fn requested_epsilon_changes_cost_and_filters_inaccurate_entries() {
    let mut document = target().document;
    document["gates"][0]["epsilon"] = json!(0.01);
    document["gates"][0]["cost"]["time"] = json!({"op":"log2inv_eps","args":[]});
    let t = Target::load(&json::canonical(&document).unwrap()).unwrap();
    for (share, time) in [("1", "0"), ("0.1", "4"), ("0.01", "7")] {
        let charge = t
            .native("cx", &["fast", "fast"], 1, 1, expr::decimal(share).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(charge.cost["time"], expr::decimal(time).unwrap());
        assert_eq!(charge.epsilon, expr::decimal("0.01").unwrap());
    }
    assert!(t
        .native(
            "cx",
            &["fast", "fast"],
            1,
            1,
            expr::decimal("0.001").unwrap()
        )
        .unwrap()
        .is_none());
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
    let e = json::parse(
        &std::fs::read_to_string(pins::isa().join("conformance/cases/E/E-aggregation/events.json"))
            .unwrap(),
    )
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
    d["imports"] = json!({"probe":{"version":"quantity-map-witness","map":{"quantity":["gates",0.125],"extra":["gates",3.5],"duration":["time",0.000001]}}});
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    let converted = t
        .import_quantities(
            "probe",
            &Bindings::from([
                (
                    "quantity".into(),
                    expr::decimal("1000000000000000000000").unwrap(),
                ),
                ("extra".into(), expr::decimal("7").unwrap()),
                ("duration".into(), expr::decimal("2000001").unwrap()),
            ]),
        )
        .unwrap();
    assert_eq!(
        converted["gates"],
        expr::decimal("125000000000000000024.5").unwrap()
    );
    assert_eq!(converted["time"], expr::decimal("2.000001").unwrap());
    for (model, quantity, value, message) in [
        ("missing", "quantity", "1", "unknown imported model"),
        ("probe", "absent", "1", "unmapped imported quantity absent"),
        ("probe", "quantity", "-1", "nonnegative quantities required"),
    ] {
        let error = t
            .import_quantities(
                model,
                &Bindings::from([(quantity.into(), expr::decimal(value).unwrap())]),
            )
            .unwrap_err();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn parameters_and_predicate_limits_are_checked() {
    let mut d = target().document;
    d["gates"][0]["op"] = json!("rz");
    d["gates"][0]["params"] = json!(["theta"]);
    d["gates"][0]["controls"] = json!({"min":0,"max":"unbounded"});
    d["capabilities"]["max_predicate_depth"] = json!(0);
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert!(t
        .expand("rz", &["fast"], 0, 0, expr::decimal("0").unwrap(), None)
        .is_err());
    assert!(t
        .expand(
            "rz",
            &["fast", "fast"],
            0,
            1,
            expr::decimal("0").unwrap(),
            Some(expr::decimal("0.5").unwrap())
        )
        .is_err());
    assert!(t
        .expand(
            "rz",
            &["fast"],
            0,
            0,
            expr::decimal("0").unwrap(),
            Some(expr::decimal("0.5").unwrap())
        )
        .is_ok());
}

#[test]
fn cz_scheme_uses_two_targets_and_zero_own_controls() {
    let mut d = target().document;
    d["gates"][0]["op"] = json!("cz");
    d["gates"][0]["controls"] = json!({"min":0,"max":0});
    // A SWAP-shaped alias is only an expansion probe, not a correctness claim.
    d["schemes"] = json!([{"name":"two-target-probe","realises":{"op":"swap","controls":{"min":0,"max":0}},"ancillae":{"count":0,"state":"clean","class":"fast"},"template":[{"gate":"cz","wires":["t","t2"]}]}]);
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    let leaves = t
        .expand(
            "swap",
            &["fast", "fast"],
            0,
            0,
            expr::decimal("0").unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(leaves[0].wires, vec![0, 1]);
    d["schemes"][0]["realises"]["op"] = json!("cz");
    d["gates"][0]["op"] = json!("swap");
    d["schemes"][0]["template"][0]["gate"] = json!("swap");
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    assert!(t
        .expand(
            "cz",
            &["fast", "fast"],
            0,
            0,
            expr::decimal("0").unwrap(),
            None
        )
        .is_ok());
}

#[test]
fn repeat_indices_can_be_signed_in_parameter_expressions() {
    let mut d = target().document;
    d["gates"][0]["op"] = json!("rz");
    d["gates"][0]["params"] = json!(["theta"]);
    d["gates"][0]["controls"] = json!({"min":0,"max":0});
    d["schemes"] = json!([{"name":"signed-repeat","realises":{"op":"rx","controls":{"min":0,"max":0},"params":["theta"]},"ancillae":{"count":0,"state":"clean","class":"fast"},"template":[{"gate":"rz","wires":["t"],"params":[{"var":"i"}],"repeat":{"var":"i","from":-1,"to":1}}]}]);
    let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
    let leaves = t
        .expand(
            "rx",
            &["fast"],
            0,
            0,
            expr::decimal("0").unwrap(),
            Some(expr::decimal("0.5").unwrap()),
        )
        .unwrap();
    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0].params[0], expr::decimal("-1").unwrap());
}

fn scheme_target() -> Target {
    let mut document =
        json::parse(include_str!("../../../targets/ideal-quantum-circuit.json")).unwrap();
    document["gates"] = json!([{"op":"cx","classes":["*"],"controls":{"min":1,"max":1},"epsilon":0,"cost":{"time":1}}]);
    document["schemes"] = json!([{"name":"wire-probe","realises":{"op":"swap","controls":{"min":0,"max":0}},"ancillae":{"count":0,"class":"data","state":"clean"},"template":[{"gate":"cx","wires":["t","t2"]}]}]);
    Target::load(&json::canonical(&document).unwrap()).unwrap()
}

#[test]
fn malformed_ancilla_class_returns_an_error_without_panicking() {
    let mut t = scheme_target();
    t.document["schemes"][0]["ancillae"]["class"] = json!(null);
    let error = t
        .expand(
            "swap",
            &["data", "data"],
            0,
            0,
            expr::decimal("0").unwrap(),
            None,
        )
        .unwrap_err();
    assert!(error.contains("ancillae.class"), "{error}");
}

#[test]
fn malformed_template_gate_returns_an_error_without_panicking() {
    let mut t = scheme_target();
    t.document["schemes"][0]["template"][0]["gate"] = json!(null);
    let error = t
        .expand(
            "swap",
            &["data", "data"],
            0,
            0,
            expr::decimal("0").unwrap(),
            None,
        )
        .unwrap_err();
    assert!(error.contains("template gate"), "{error}");
}

#[test]
fn malformed_template_wire_returns_an_error_without_panicking() {
    for wire in ["", "a", "z[0]", "c[]", "c[-1]", "a[0]"] {
        let mut t = scheme_target();
        t.document["schemes"][0]["template"][0]["wires"][0] = json!(wire);
        let error = t
            .expand(
                "swap",
                &["data", "data"],
                0,
                0,
                expr::decimal("0").unwrap(),
                None,
            )
            .unwrap_err();
        assert!(error.contains("§8.3"), "{wire}: {error}");
    }
}

#[test]
fn a_live_class_without_vertices_names_the_missing_limit() {
    let mut document = scheme_target().document;
    document["topology"] = json!({"vertices":[],"edges":[]});
    let t = Target::load(&json::canonical(&document).unwrap()).unwrap();
    let error = t.aggregate(&json!({"leaves":[{"kind":"classical","start":0,"duration":1}],"live_intervals":[{"wire":0,"class":"data","start":0,"end":1}]})).unwrap_err();
    assert_eq!(error, "§8.3: class data has no declared vertex limit");
}

/// Independent quarter-second sampling, rather than the implementation's
/// event-endpoint sweep. Borrowers replace owners and rates differ by class.
#[test]
fn aggregation_matches_a_discrete_oracle_with_borrowed_wires() {
    use std::collections::BTreeMap;
    let mut document = json::parse(include_str!(
        "../../../targets/superconducting-grid-illustrative.json"
    ))
    .unwrap();
    document["resources"]["qubit_second"]["rate"] = json!({"data":3,"buffer":5});
    document["capabilities"]["classical_step_time"] = json!(1);
    let t = Target::load(&json::canonical(&document).unwrap()).unwrap();
    for scenario in 0..32 {
        let mut intervals = Vec::new();
        let mut borrows = Vec::new();
        for wire in 0..3 {
            let owner = if (scenario + wire) % 2 == 0 {
                "data"
            } else {
                "buffer"
            };
            intervals.push(json!({"wire":wire,"class":owner,"start":0,"end":2}));
            let start = (scenario + wire * 3) % 7;
            let end = start + 1 + (scenario * 3 + wire) % (8 - start);
            intervals.push(json!({"wire":wire,"class":"buffer","start":start as f64/4.0,"end":end as f64/4.0,"borrowed":true}));
            borrows.push((owner, start, end));
        }
        let mut occupancy = BTreeMap::from([
            ("data", expr::decimal("0").unwrap()),
            ("buffer", expr::decimal("0").unwrap()),
        ]);
        let mut peaks = BTreeMap::from([("data", 0), ("buffer", 0)]);
        for tick in 0..8 {
            let mut live = BTreeMap::from([("data", 0), ("buffer", 0)]);
            for (owner, start, end) in &borrows {
                let borrowed = *start <= tick && tick < *end;
                let class = if borrowed { "buffer" } else { owner };
                *live.get_mut(class).unwrap() += 1;
                let rate = expr::decimal(if class == "data" { "3" } else { "5" }).unwrap();
                let weight = expr::decimal(if borrowed { "0.25" } else { "1" }).unwrap();
                *occupancy.get_mut(class).unwrap() += rate * weight / expr::decimal("4").unwrap();
            }
            for (class, n) in live {
                *peaks.get_mut(class).unwrap() = peaks[class].max(n);
            }
        }
        let result = t.aggregate(&json!({"leaves":[{"kind":"classical","start":0,"duration":1,"cost":{"gate_count":3}},{"kind":"effect","start":1,"duration":1,"cost":{"gate_count":7}}],"live_intervals":intervals})).unwrap();
        assert_eq!(result["time"]["total"], expr::decimal("2").unwrap());
        assert_eq!(result["gate_count"]["total"], expr::decimal("10").unwrap());
        for class in ["data", "buffer"] {
            assert_eq!(
                result["qubit_second"][class], occupancy[class],
                "scenario {scenario}, {class}"
            );
            assert_eq!(
                result["peak_qubits"][class],
                expr::Rational::from_integer(peaks[class].into()),
                "scenario {scenario}, {class}"
            );
        }
    }
}
