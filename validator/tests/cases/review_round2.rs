//! Round-2 regressions for docs/reviews/2026-11-03-targets-batch.md (ISA 0.9 §4.4, §8, §11).
use bennett_targets_validator::{expr, importers, json, Target};
use serde_json::{json, Value};

fn base() -> Value {
    json::parse(include_str!("../../../targets/ideal-all-to-all.json")).unwrap()
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

/// §8.1 numbers are exact rationals from their decimal text, so an
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

/// §8.3 "Each live wire occupies one vertex of its resolved class";
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

/// §8.3 K86 "Closed-schema target readers MUST be upgraded for
/// realisation_model". The accepted field set is {version,kind,arity,basis,
/// parameter_bindings,control_models,data}; operand_map
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

/// §8.1 decimal text; "1.2.3" is not a number.
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

/// §4.4 forms fix every opcode's operand arity (x a; cx c t; ...).
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

#[test]
fn zero_epsilon_expressions_reject_contradictory_native_models_at_load() {
    for epsilon in [
        json!({"var":"k"}),
        json!({"op":"-","args":[{"var":"k"},{"var":"k"}]}),
        json!({"table":"accuracy","index":"k"}),
    ] {
        let mut d = base();
        d["tables"]["accuracy"] = json!([0]);
        d["gates"][0]["epsilon"] = epsilon.clone();
        d["gates"][0]["realisation_model"] = x_model();
        let t = load(&d).unwrap();
        assert_eq!(
            t.native("x", &["data"], 0, 0, zero())
                .unwrap()
                .unwrap()
                .epsilon,
            zero()
        );
        d["gates"][0]["realisation_model"]["data"]["matrix"][0][1][0][0] = json!("-1");
        d["gates"][0]["realisation_model"]["data"]["matrix"][1][0][0][0] = json!("-1");
        let error = load(&d).unwrap_err();
        assert!(
            error.contains("contradicts") && error.contains("zero-epsilon"),
            "{epsilon}: {error}"
        );
    }
}

#[test]
fn native_models_are_checked_at_each_resolved_epsilon_and_binding() {
    for off_diagonal in [
        json!(["-1", "1"]),
        json!({"op":"-","args":[{"var":"own_controls"},["1","1"]]}),
    ] {
        let mut d = base();
        let mut model = x_model();
        model["parameter_bindings"] = json!({"own_controls":"k"});
        model["data"]["matrix"][0][1][0] = off_diagonal.clone();
        model["data"]["matrix"][1][0][0] = off_diagonal;
        d["gates"][0]["realisation_model"] = model;
        d["gates"][0]["epsilon"] = json!({"op":"*","args":[2,{"var":"c"}]});
        let t = load(&d).unwrap();
        let error = t.native("x", &["data"], 0, 0, zero()).unwrap_err();
        assert!(error.contains("contradicts"), "{error}");
        let error = t.expand("x", &["data"], 0, 0, zero(), None).unwrap_err();
        assert!(error.contains("contradicts"), "{error}");
        // Controlled X and controlled -X have operator distance 2. This is a
        // legal positive-epsilon context, even though c=0 must reject.
        let charge = t
            .native("x", &["data"; 2], 0, 1, expr::decimal("2").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(charge.epsilon, expr::decimal("2").unwrap());
    }
}

#[test]
fn exact_resolved_model_context_does_not_bind_eps() {
    let mut d = base();
    let mut model = x_model();
    model["parameter_bindings"]["requested_accuracy"] = json!("eps");
    d["gates"][0]["realisation_model"] = model;
    d["gates"][0]["epsilon"] = json!({"var":"c"});
    let t = load(&d).unwrap();
    let error = t
        .native("x", &["data"], 0, 0, expr::decimal("1").unwrap())
        .unwrap_err();
    assert!(
        error.contains("eps is bound only where evaluated epsilon is positive"),
        "{error}"
    );
    assert_eq!(
        t.native("x", &["data"; 2], 0, 1, expr::decimal("1").unwrap())
            .unwrap()
            .unwrap()
            .epsilon,
        expr::decimal("1").unwrap()
    );
}

#[test]
fn zero_epsilon_control_expressions_check_models_at_resolution() {
    for (op, k, c) in [("x", 0, 0), ("cx", 1, 1), ("ccx", 2, 2), ("mcx", 2, 2)] {
        let mut d = base();
        let i = gate(&d, op);
        d["gates"][i]["epsilon"] = json!({"op":"-","args":[{"var":"c"},{"var":"c"}]});
        d["gates"][i]["realisation_model"] = x_model();
        let t = load(&d).unwrap();
        let classes = vec!["data"; c + 1];
        assert_eq!(
            t.native(op, &classes, k, c, zero())
                .unwrap()
                .unwrap()
                .epsilon,
            zero()
        );
        // The document can be mutated by library callers after schema loading.
        let mut t = t;
        t.document["gates"][i]["realisation_model"]["data"]["matrix"][0][1][0][0] = json!("-1");
        t.document["gates"][i]["realisation_model"]["data"]["matrix"][1][0][0][0] = json!("-1");
        let error = t.native(op, &classes, k, c, zero()).unwrap_err();
        assert!(error.contains("contradicts"), "{op}: {error}");
        t.document["gates"][i]["controls"] = json!({"min":c,"max":c});
        let error = load(&t.document).unwrap_err();
        assert!(error.contains("contradicts"), "{op}: {error}");
    }
}

#[test]
fn maximal_control_counts_return_arity_errors_without_panicking() {
    let t = load(&base()).unwrap();
    for (op, c) in [
        ("x", usize::MAX),
        ("swap", usize::MAX),
        ("swap", usize::MAX - 1),
    ] {
        let native_error = t.native(op, &["data"], 0, c, zero()).unwrap_err();
        let expand_error = t.expand(op, &["data"], 0, c, zero(), None).unwrap_err();
        for error in [native_error, expand_error] {
            assert!(
                error.contains("host limit") && error.contains("operand count"),
                "{op}: {error}"
            );
            assert!(error.contains(&format!("c={c}")), "{error}");
        }
    }
}

fn angle_identity_model(sign: &str) -> Value {
    let mut model = x_model();
    model["parameter_bindings"] = json!({"angle":"theta"});
    let zero = json!(["0", "1"]);
    let diagonal = json!([[sign, "1"], zero]);
    let off_diagonal = json!([{"op":"-","args":[{"var":"angle"},{"var":"angle"}]},zero]);
    model["data"]["matrix"] = json!([[diagonal, off_diagonal], [off_diagonal, diagonal]]);
    model
}

fn rotation_document(op: &str, model: Value) -> Value {
    let mut document = base();
    document["gates"] = json!([{
        "op":op, "classes":["*"], "controls":{"min":0,"max":"unbounded"},
        "params":["theta"], "epsilon":{"op":"-","args":[{"var":"c"},{"var":"c"}]},
        "cost":{"time":1}, "realisation_model":model
    }]);
    document["schemes"] = json!([]);
    document
}

fn add_ry_scheme(document: &mut Value) {
    let base = base();
    for op in ["s", "sdg"] {
        document["gates"]
            .as_array_mut()
            .unwrap()
            .push(base["gates"][gate(&base, op)].clone());
    }
    // S Rx(2 theta/3) Rx(theta/3) S† = Ry(theta). Execution is
    // right to left; both repeated leaves must bind their own evaluated angle.
    document["schemes"] = json!([{
        "name":"ry-via-rx", "realises":{"op":"ry","controls":{"min":0,"max":0},"params":["theta"]},
        "ancillae":{"count":0,"class":"data","state":"clean"},
        "template":[
            {"gate":"sdg","wires":["t"]},
            {"gate":"rx","wires":["t"],"params":[{"op":"/","args":[
                {"op":"*","args":[{"var":"theta"},{"op":"+","args":[{"var":"i"},1]}]},3
            ]}],"repeat":{"var":"i","from":0,"to":2}},
            {"gate":"s","wires":["t"]}
        ]
    }]);
}

#[test]
fn direct_rotation_model_binds_the_actual_angle() {
    // §4.4: exp(0) = +I for every axis; -I differs by operator distance 2.
    for op in ["rx", "ry", "rz"] {
        let t = load(&rotation_document(op, angle_identity_model("-1"))).unwrap();
        let error = t
            .expand(op, &["data"], 0, 0, zero(), Some(zero()))
            .unwrap_err();
        assert!(
            error.contains("contradicts") && error.contains(op),
            "{error}"
        );
    }
}

#[test]
fn controlled_rotation_model_binds_the_actual_angle() {
    // At theta=0 the ideal is I4; controlled -I is diag(1,1,-1,-1).
    for op in ["rx", "ry", "rz"] {
        let t = load(&rotation_document(op, angle_identity_model("-1"))).unwrap();
        let error = t
            .expand(op, &["data"; 2], 0, 1, zero(), Some(zero()))
            .unwrap_err();
        assert!(
            error.contains("contradicts") && error.contains(op),
            "{error}"
        );
    }
}

#[test]
fn template_rotation_model_binds_the_evaluated_leaf_angle() {
    let mut document = rotation_document("rx", angle_identity_model("-1"));
    add_ry_scheme(&mut document);
    let error = load(&document)
        .unwrap()
        .expand("ry", &["data"], 0, 0, zero(), Some(zero()))
        .unwrap_err();
    assert!(
        error.contains("contradicts") && error.contains("rx"),
        "{error}"
    );
}

#[test]
fn exact_identity_rotation_models_remain_valid_at_zero() {
    for op in ["rx", "ry", "rz"] {
        let t = load(&rotation_document(op, angle_identity_model("1"))).unwrap();
        for c in [0, 1] {
            let leaves = t
                .expand(op, &vec!["data"; c + 1], 0, c, zero(), Some(zero()))
                .unwrap();
            assert_eq!(leaves.len(), 1);
            assert_eq!(leaves[0].params, vec![zero()]);
            assert_eq!(leaves[0].charge.epsilon, zero());
        }
    }
    let mut document = rotation_document("rx", angle_identity_model("1"));
    add_ry_scheme(&mut document);
    let leaves = load(&document)
        .unwrap()
        .expand("ry", &["data"], 0, 0, zero(), Some(zero()))
        .unwrap();
    assert_eq!(leaves.len(), 4);
    assert_eq!(leaves[1].params, vec![zero()]);
    assert_eq!(leaves[2].params, vec![zero()]);
}

fn analytical_rotation_model(op: &str) -> Value {
    let mut model = x_model();
    model["parameter_bindings"] = json!({"angle":"theta"});
    // Hand-derived from exp(-i theta P/2), P²=I (§4.4): C I - i S P.
    let half = json!({"op":"/","args":[{"var":"angle"},["2","1"]]});
    let c = json!({"op":"cos","args":[half]});
    let s = json!({"op":"sin","args":[half]});
    let zero = json!(["0", "1"]);
    let minus_s = json!({"op":"-","args":[zero,s]});
    model["data"]["matrix"] = match op {
        "rx" => json!([[[c, zero], [zero, minus_s]], [[zero, minus_s], [c, zero]]]),
        "ry" => json!([[[c, zero], [minus_s, zero]], [[s, zero], [c, zero]]]),
        "rz" => json!([[[c, minus_s], [zero, zero]], [[zero, zero], [c, s]]]),
        _ => panic!("not a rotation: {op}"),
    };
    model
}

#[test]
fn nontrivial_rotation_models_preserve_exact_angles_on_all_paths() {
    // theta=3/4 radians: C=cos(3/8), S=sin(3/8); controlled_U adds
    // identity on its disabled branch without changing either phase.
    let angle = expr::decimal("0.75").unwrap();
    for op in ["rx", "ry", "rz"] {
        let t = load(&rotation_document(op, analytical_rotation_model(op))).unwrap();
        for c in [0, 1] {
            let leaves = t
                .expand(op, &vec!["data"; c + 1], 0, c, zero(), Some(angle.clone()))
                .unwrap();
            assert_eq!(leaves.len(), 1);
            assert_eq!(leaves[0].op, op);
            assert_eq!(
                leaves[0].params,
                vec![expr::Rational::new(3.into(), 4.into())]
            );
            assert_eq!(leaves[0].charge.epsilon, zero());
        }
    }
    let mut document = rotation_document("rx", analytical_rotation_model("rx"));
    add_ry_scheme(&mut document);
    let leaves = load(&document)
        .unwrap()
        .expand("ry", &["data"], 0, 0, zero(), Some(angle))
        .unwrap();
    assert_eq!(
        leaves.iter().map(|l| l.op.as_str()).collect::<Vec<_>>(),
        ["sdg", "rx", "rx", "s"]
    );
    // (i+1)*3/4/3 = 1/4, 1/2 at i=0,1, giving Ry(3/4).
    assert_eq!(
        leaves[1].params,
        vec![expr::Rational::new(1.into(), 4.into())]
    );
    assert_eq!(
        leaves[2].params,
        vec![expr::Rational::new(1.into(), 2.into())]
    );
    assert!(leaves.iter().all(|l| l.charge.epsilon == zero()));
}

#[test]
fn template_params_require_one_resolved_angle() {
    let mut document = rotation_document("rx", analytical_rotation_model("rx"));
    add_ry_scheme(&mut document);
    for params in [json!([]), json!([0, 1])] {
        document["schemes"][0]["template"][1]["params"] = params;
        let error = load(&document)
            .and_then(|t| t.expand("ry", &["data"], 0, 0, zero(), Some(zero())))
            .unwrap_err();
        assert!(
            error.contains("params") && error.contains("one angle"),
            "{error}"
        );
    }
}

#[test]
fn template_model_uses_the_leaf_angle_instead_of_the_scheme_angle() {
    let mut document = rotation_document("rx", analytical_rotation_model("rx"));
    add_ry_scheme(&mut document);
    let rz = rotation_document("rz", angle_identity_model("1"))["gates"][0].clone();
    document["gates"].as_array_mut().unwrap().push(rz);
    // Append Rz(theta-theta) = I to the correct Ry(theta) expansion. The
    // rational identity model agrees at leaf angle 0, not parent angle 3/4.
    document["schemes"][0]["template"]
        .as_array_mut()
        .unwrap()
        .push(json!({"gate":"rz","wires":["t"],"params":[
            {"op":"-","args":[{"var":"theta"},{"var":"theta"}]}
        ]}));
    let leaves = load(&document)
        .unwrap()
        .expand(
            "ry",
            &["data"],
            0,
            0,
            zero(),
            Some(expr::decimal("0.75").unwrap()),
        )
        .unwrap();
    assert_eq!(leaves.len(), 5);
    assert_eq!(leaves[4].op, "rz");
    assert_eq!(leaves[4].params, vec![zero()]);
    assert_eq!(leaves[4].charge.epsilon, zero());
}
