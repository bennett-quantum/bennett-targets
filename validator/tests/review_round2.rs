//! Round-2 regressions for docs/reviews/2026-11-03-targets-batch.md (ISA 0.9 §4.4, §8, §11).
use bennett_targets_validator::{expr, importers, json, Target};
use serde_json::{json, Value};

fn base() -> Value {
    json::parse(include_str!("../../targets/ideal-all-to-all.json")).unwrap()
}
fn load(d: &Value) -> Result<Target, String> {
    Target::load(&json::canonical(d).unwrap())
}
fn zero() -> expr::Rational {
    expr::decimal("0").unwrap()
}
fn azure(rate: &str, gate_time: &str) -> Result<Value, String> {
    let input = json::parse(&format!(
        r#"{{"qubitParams":{{"name":"r2","instructionSet":"GateBased","oneQubitMeasurementTime":"1 us","oneQubitGateTime":"{gate_time}","oneQubitMeasurementErrorRate":0,"oneQubitGateErrorRate":{rate}}}}}"#
    ))
    .unwrap();
    importers::azure(&load(&base()).unwrap(), &input)
}

/// b-targets-c5w: §8.1 numbers are exact rationals from their decimal text, so an
/// import either keeps the source value exactly or fails loudly; never rounds.
#[test]
fn imported_rates_are_preserved_exactly_or_rejected() {
    for rate in ["0.10000000000000001", "1e-400", "1.00000000000000000001e-5"] {
        let err = azure(rate, "20 us").unwrap_err();
        assert!(err.contains("oneQubitGateErrorRate"), "{rate}: {err}");
        assert!(err.contains("exactly"), "{rate}: {err}");
    }
    for rate in ["0.1", "1e-4", "0.001", "0"] {
        let report = azure(rate, "20 us").unwrap();
        let t = Target::load(report["canonical"].as_str().unwrap()).unwrap();
        let charge = t.native("x", &["data"], 0, 0, zero()).unwrap().unwrap();
        assert_eq!(
            charge.cost["azure.oneQubitGateErrorRate"],
            expr::decimal(rate).unwrap(),
            "{rate}"
        );
        assert_eq!(
            expr::number(&report["quantities"]["azure.oneQubitGateErrorRate"]).unwrap(),
            expr::decimal(rate).unwrap()
        );
    }
}

/// b-targets-9oc: §8.3 "Each live wire occupies one vertex of its resolved class";
/// §11 binds placements, so a fast-class leaf on a slow vertex rejects.
#[test]
fn placement_must_match_the_resolved_operand_class() {
    let mut d = base();
    d["classes"] = json!({
        "fast":{"capacity":2,"dirty_rate":1,"qubit_second":1},
        "slow":{"capacity":2,"dirty_rate":1,"qubit_second":1}
    });
    d["gates"] = json!([
        {"op":"x","classes":["fast"],"controls":{"min":0,"max":0},"epsilon":0,"cost":{"time":1}},
        {"op":"x","classes":["slow"],"controls":{"min":0,"max":0},"epsilon":0,"cost":{"time":100}},
        {"op":"cx","classes":["*"],"controls":{"min":1,"max":1},"epsilon":0,"cost":{"time":1}}
    ]);
    d["schemes"] = json!([]);
    d["topology"] = json!({"vertices":[{"class":"slow"},{"class":"fast"}],"edges":[[0,1]]});
    let t = load(&d).unwrap();
    let fast = t.expand("x", &["fast"], 0, 0, zero(), None).unwrap();
    assert_eq!(fast[0].charge.cost["time"], expr::decimal("1").unwrap());
    let err = t.check_leaves(&fast, &[0]).unwrap_err();
    assert!(err.contains("class"), "{err}");
    t.check_leaves(&fast, &[1]).unwrap();
    let slow = t.expand("x", &["slow"], 0, 0, zero(), None).unwrap();
    assert_eq!(slow[0].charge.cost["time"], expr::decimal("100").unwrap());
    t.check_leaves(&slow, &[0]).unwrap();
    assert!(t.check_leaves(&slow, &[1]).is_err());
    // Mixed-class operands: each wire is checked against its own vertex.
    let cx = t
        .expand("cx", &["slow", "fast"], 1, 1, zero(), None)
        .unwrap();
    t.check_leaves(&cx, &[0, 1]).unwrap();
    assert!(t.check_leaves(&cx, &[1, 0]).is_err());
    // Distinct wires of one expansion occupy distinct vertices.
    let mut d2 = d.clone();
    d2["topology"] = json!({"vertices":[{"class":"fast"},{"class":"fast"},{"class":"fast"}],"edges":[[0,1],[0,2],[1,2]]});
    d2["schemes"] = json!([{
        "name":"x-via-ancilla","realises":{"op":"x","controls":{"min":1,"max":1}},
        "ancillae":{"count":1,"class":"fast","state":"clean"},
        "template":[{"gate":"cx","wires":["c[0]","a[0]"]},{"gate":"cx","wires":["a[0]","t"]}]
    }]);
    let t2 = load(&d2).unwrap();
    let leaves = t2
        .expand("x", &["fast", "fast"], 0, 1, zero(), None)
        .unwrap();
    t2.check_leaves(&leaves, &[0, 1, 2]).unwrap();
    // Wires 0 and 1 share no leaf, so only per-expansion distinctness catches this.
    let err = t2.check_leaves(&leaves, &[0, 0, 1]).unwrap_err();
    assert!(err.contains("distinct"), "{err}");
}

/// The published K86 X base model (§8.3), verbatim.
fn x_model() -> Value {
    json!({
        "version":"bennett.native-model/1", "kind":"unitary", "arity":1,
        "basis":["t"], "parameter_bindings":{"k":"k","c":"c"},
        "control_models":{"kind":"controlled_U","controls":"c","disabled":"identity"},
        "data":{"matrix":[
            [[["0","1"],["0","1"]],[["1","1"],["0","1"]]],
            [[["1","1"],["0","1"]],[["0","1"],["0","1"]]]
        ]}
    })
}
fn gate(d: &Value, op: &str) -> usize {
    d["gates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|g| g["op"] == op)
        .unwrap()
}
fn with_model(op: &str, model: Value) -> Result<Target, String> {
    let mut d = base();
    let i = gate(&d, op);
    d["gates"][i]["realisation_model"] = model;
    load(&d)
}

/// b-targets-z7b: §8.3 K86 "Closed-schema target readers MUST be upgraded for
/// realisation_model". The accepted field set is {version,kind,arity,basis,
/// parameter_bindings,control_models,data}; operand_map (b-isa-spec-wk0k, open)
/// is tolerated only as a verified operand permutation.
#[test]
fn published_native_realisation_model_loads_and_is_verified() {
    // Accepted field set, all admitted c (controlled_U).
    let t = with_model("x", x_model()).unwrap();
    assert!(t.document["gates"][0]
        .get("realisation_model")
        .is_some_and(|m| m == &x_model()));
    // The reviewer's exact probe: X at max 0 controls with resolved operand_map [0].
    let mut d = base();
    let mut m = x_model();
    m["operand_map"] = json!([0]);
    d["gates"][0]["realisation_model"] = m;
    d["gates"][0]["controls"]["max"] = json!(0);
    load(&d).unwrap();
    // §8.3: cx, own control a, inherited q: entry order [a,q,t] -> checker [q,a,t].
    let mut m = x_model();
    m["operand_map"] = json!([1, 0, 2]);
    with_model("cx", m).unwrap();
    // Model bytes enter target_hash.
    assert_ne!(
        with_model("x", x_model()).unwrap().hash,
        load(&base()).unwrap().hash
    );

    let mut bad = Vec::new();
    // −X contradicts the exact zero-epsilon contract (§8.3 last K86 paragraph).
    let mut m = x_model();
    m["data"]["matrix"][0][1][0][0] = json!("-1");
    m["data"]["matrix"][1][0][0][0] = json!("-1");
    bad.push(("x", m, "contradicts"));
    // A rational-only matrix can never be the exact h.
    bad.push(("h", x_model(), "contradicts"));
    let mut m = x_model();
    m["operand_map"] = json!([0, 0]);
    bad.push(("x", m, "operand_map"));
    let mut m = x_model();
    m["operand_map"] = json!([0]);
    bad.push(("cx", m, "operand_map")); // cx admits c >= 1 only
    let mut m = x_model();
    m["unknown"] = json!(0);
    bad.push(("x", m, "unknown field"));
    let mut m = x_model();
    m["version"] = json!("bennett.native-model/2");
    bad.push(("x", m, "unsupported"));
    let mut m = x_model();
    m["arity"] = json!(2);
    bad.push(("x", m, "arity"));
    let mut m = x_model();
    m["data"]["matrix"][0][0][0] = json!(["2", "4"]);
    bad.push(("x", m, "reduced"));
    let mut m = x_model();
    m["data"]["matrix"].as_array_mut().unwrap().pop();
    bad.push(("x", m, "dimension"));
    let mut m = x_model();
    m["parameter_bindings"]["eps"] = json!("eps");
    bad.push(("x", m, "eps"));
    let mut m = x_model();
    m["data"]["matrix"][0][0][0] = json!({"var":"theta"});
    bad.push(("x", m, "theta"));
    let mut m = x_model();
    m["kind"] = json!("kraus");
    bad.push(("x", m, "unitary"));
    let mut m = x_model();
    m["operand_map"] = json!([0, 1]);
    bad.push(("ccx", m, "operand_map")); // c=1 cannot hold ccx's two own controls
    for (op, model, needle) in bad {
        let err = with_model(op, model.clone()).unwrap_err();
        assert!(err.contains(needle), "{op} {model}: {err}");
    }
}

/// b-targets-9vu: §8.1 decimal text; "1.2.3" is not a number.
#[test]
fn malformed_decimal_text_rejects() {
    for bad in [
        "1.2.3", "1..2", ".5", "1.", "+1", "--1", "1e", "1e+", "0x10", "1_0", "", "-", "01",
        "1.2e3.4", " 1",
    ] {
        assert!(expr::decimal(bad).is_err(), "{bad:?}");
    }
    for (good, n, d) in [
        ("0", 0, 1),
        ("-0.5", -1, 2),
        ("1.25e2", 125, 1),
        ("1E-2", 1, 100),
        ("1e+2", 100, 1),
    ] {
        assert_eq!(
            expr::decimal(good).unwrap(),
            expr::Rational::new(n.into(), d.into()),
            "{good}"
        );
    }
    let err = azure("0", "1.2.3 us").unwrap_err();
    assert!(err.contains("§8.1"), "{err}");
}

/// b-targets-4z8: §4.4 forms fix every opcode's operand arity (x a; cx c t; ...).
#[test]
fn native_operand_arity_is_checked() {
    let t = load(&base()).unwrap();
    for (op, classes, k, c) in [
        ("x", vec![], 0, 0),
        ("x", vec!["data", "data"], 0, 0),
        ("cx", vec!["data"], 1, 1),
        ("cx", vec!["data", "data"], 0, 1),
        ("ccx", vec!["data", "data"], 2, 2),
        ("swap", vec!["data"], 0, 0),
        ("mcx", vec!["data", "data"], 2, 2),
    ] {
        let err = t.expand(op, &classes, k, c, zero(), None).unwrap_err();
        assert!(err.contains("§4.4"), "{op} {classes:?}: {err}");
        assert!(t.native(op, &classes, k, c, zero()).is_err(), "{op}");
    }
    // Effective controls add operands: x under one inherited control has two.
    assert_eq!(
        t.expand("x", &["data", "data"], 0, 1, zero(), None)
            .unwrap()[0]
            .wires,
        vec![0, 1]
    );
    assert_eq!(
        t.expand("mcx", &["data"; 3], 2, 2, zero(), None).unwrap()[0].wires,
        vec![0, 1, 2]
    );
    // A scheme template leaf is held to the same arity.
    let mut d = base();
    d["gates"] = json!([{
        "op":"cx", "classes":["*"], "controls":{"min":1,"max":1},
        "epsilon":0, "cost":{"time":1,"gate_count":1}
    }]);
    d["schemes"] = json!([{
        "name":"invalid-unary-cx", "realises":{"op":"swap","controls":{"min":0,"max":0}},
        "ancillae":{"count":0,"class":"data","state":"clean"},
        "template":[{"gate":"cx","wires":["t"]}]
    }]);
    let err = load(&d)
        .and_then(|t| t.expand("swap", &["data", "data"], 0, 0, zero(), None))
        .unwrap_err();
    assert!(err.contains("§4.4"), "{err}");
}
