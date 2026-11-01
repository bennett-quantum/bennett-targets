//! ISA 0.9 §8 deltas: K9 EPS-DECADE and K14 topology, from the published text.
use bennett_targets_validator::{
    expr::{self, Bindings},
    json, Target,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bennett-isa-spec/conformance/cases/E")
}
fn grid_text() -> String {
    std::fs::read_to_string(cases().join("E-topology-grid-09/target.json")).unwrap()
}
fn grid() -> Value {
    json::parse(&grid_text()).unwrap()
}
fn load(d: &Value) -> Result<Target, String> {
    Target::load(&json::canonical(d).unwrap())
}
fn eps(text: &str) -> Bindings {
    Bindings::from([("eps".to_string(), expr::decimal(text).unwrap())])
}
fn lookup(n: usize) -> (Value, Value) {
    let values: Vec<usize> = (0..n).map(|i| 100 + i).collect();
    (
        json!({"table":"t","index":"eps_decade"}),
        json!({ "t": values }),
    )
}

/// The EPS-DECADE witness of §8.1, verbatim.
#[test]
fn eps_decade_published_witness() {
    let (e, tables) = lookup(8);
    for (text, index) in [
        ("1", 0),
        ("0.1", 1),
        ("0.1001", 1),
        ("0.0999", 2),
        ("0.001", 3),
        ("0.000999", 4),
    ] {
        let got = expr::eval(&e, &eps(text), &tables).unwrap();
        assert_eq!(
            got,
            expr::Rational::from_integer((100 + index).into()),
            "{text}"
        );
    }
}

#[test]
fn eps_decade_out_of_array_and_three_entry_boundary() {
    let (e, two) = lookup(2);
    let err = expr::eval(&e, &eps("0.0999"), &two).unwrap_err();
    assert!(err.contains("outside the array"), "{err}");
    let (e, three) = lookup(3);
    assert_eq!(
        expr::eval(&e, &eps("0.0999"), &three).unwrap(),
        expr::Rational::from_integer(102.into())
    );
}

#[test]
fn eps_decade_domain_rejects_without_fallback() {
    let (e, tables) = lookup(8);
    for bad in ["0", "1.5"] {
        let err = expr::eval(&e, &eps(bad), &tables).unwrap_err();
        assert!(err.contains("(0,1]"), "{bad}: {err}");
    }
    let unbound = Bindings::from([("k".to_string(), expr::decimal("1").unwrap())]);
    assert!(expr::eval(&e, &unbound, &tables)
        .unwrap_err()
        .contains("§8.1"));
}

/// `eps_decade` must not silently keep reading `k`, the 0.8 validator's path.
#[test]
fn eps_decade_does_not_read_k() {
    let (e, tables) = lookup(8);
    let mut env = eps("0.01");
    env.insert("k".into(), expr::decimal("5").unwrap());
    assert_eq!(
        expr::eval(&e, &env, &tables).unwrap(),
        expr::Rational::from_integer(102.into())
    );
}

fn with_table_cost(epsilon: Value, cost: Value) -> Value {
    let mut d = grid();
    d["tables"] = json!({"t":[1,2,3,4,5]});
    d["gates"][0]["epsilon"] = epsilon;
    d["gates"][0]["cost"]["time"] = cost;
    d
}

#[test]
fn eps_decade_cost_charges_requested_share_not_achieved_epsilon() {
    let d = with_table_cost(json!(0.001), json!({"table":"t","index":"eps_decade"}));
    let t = load(&d).unwrap();
    let c = t
        .native(
            "cx",
            &["fast", "fast"],
            1,
            1,
            expr::decimal("0.01").unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(c.epsilon, expr::decimal("0.001").unwrap());
    assert_eq!(c.cost["time"], expr::decimal("3").unwrap());
}

#[test]
fn exact_entry_must_not_use_eps_decade_even_through_a_table() {
    let direct = with_table_cost(json!(0), json!({"table":"t","index":"eps_decade"}));
    assert!(load(&direct).unwrap_err().contains("epsilon 0"));
    // A table lookup that evaluates to an achieved epsilon of zero is exact too.
    let mut indirect = with_table_cost(
        json!({"table":"z","index":"k"}),
        json!({"op":"+","args":[1,{"table":"t","index":"eps_decade"}]}),
    );
    indirect["tables"]["z"] = json!([0, 0]);
    let t = load(&indirect).unwrap();
    let err = t
        .native("cx", &["fast", "fast"], 1, 1, expr::decimal("0.1").unwrap())
        .unwrap_err();
    assert!(err.contains("epsilon 0"), "{err}");
}

#[test]
fn eps_decade_only_where_eps_is_bound() {
    let mut d = grid();
    d["tables"] = json!({"t":[1]});
    d["qram"] = json!({
        "load":{"cost":{"time":{"table":"t","index":"eps_decade"}}},
        "store":{"cost":{"time":1}},
        "exch":{"cost":{"time":1}}
    });
    assert!(load(&d).unwrap_err().contains("§8.1"));
}

#[test]
fn published_topology_witness_loads_with_its_published_hash() {
    let t = Target::load(&grid_text()).unwrap();
    assert_eq!(
        t.hash,
        "960cee71e88da141cc6c2b13cab3f28fce1d50c006e67446aadb592e2bfd0882"
    );
}

#[test]
fn topology_schema_rejections() {
    let vertex = json!({"class":"fast"});
    let bad: Vec<(&str, Value)> = vec![
        ("unknown top", json!({"vertices":[],"edges":[],"x":1})),
        ("missing edges", json!({"vertices":[]})),
        (
            "unknown vertex field",
            json!({"vertices":[{"class":"fast","x":0}],"edges":[]}),
        ),
        (
            "undeclared class",
            json!({"vertices":[{"class":"none"}],"edges":[]}),
        ),
        (
            "self-loop",
            json!({"vertices":[vertex,vertex],"edges":[[0,0]]}),
        ),
        (
            "reversed",
            json!({"vertices":[vertex,vertex],"edges":[[1,0]]}),
        ),
        (
            "out of range",
            json!({"vertices":[vertex,vertex],"edges":[[0,2]]}),
        ),
        (
            "duplicate",
            json!({"vertices":[vertex,vertex],"edges":[[0,1],[0,1]]}),
        ),
        (
            "unsorted",
            json!({"vertices":[vertex,vertex,vertex],"edges":[[1,2],[0,1]]}),
        ),
        (
            "not a pair",
            json!({"vertices":[vertex,vertex],"edges":[[0,1,1]]}),
        ),
        (
            "fractional",
            json!({"vertices":[vertex,vertex],"edges":[[0.5,1]]}),
        ),
    ];
    for (name, topology) in bad {
        let mut d = grid();
        d["topology"] = topology;
        let err = load(&d).unwrap_err();
        assert!(err.contains("§8.3"), "{name}: {err}");
    }
}

#[test]
fn adjacency_is_the_published_outcome() {
    let t = Target::load(&grid_text()).unwrap();
    t.check_placement(&[0, 1]).unwrap();
    let err = t.check_placement(&[0, 3]).unwrap_err();
    assert!(
        err.contains("E_SCHEDULE_INFEASIBLE") && err.contains("§17 item 15"),
        "{err}"
    );
    t.check_placement(&[3]).unwrap(); // single-wire gates need no edge
    assert!(t.check_placement(&[1, 1]).unwrap_err().contains("distinct"));
    assert!(t.check_placement(&[0, 9]).unwrap_err().contains("§8.3"));
}

/// Every pair must be joined, including effective control wires: 0-1-3 is a
/// path, not a triangle.
#[test]
fn every_operand_pair_needs_an_edge() {
    let t = Target::load(&grid_text()).unwrap();
    let err = t.check_placement(&[0, 1, 3]).unwrap_err();
    assert!(err.contains("0") && err.contains("3"), "{err}");
}

#[test]
fn absent_topology_is_all_to_all() {
    let mut d = grid();
    d.as_object_mut().unwrap().remove("topology");
    let t = load(&d).unwrap();
    t.check_placement(&[0, 3]).unwrap();
    t.check_placement(&[1000, 2000, 3000]).unwrap();
    assert!(t.check_placement(&[7, 7]).unwrap_err().contains("distinct"));
}

/// §11: lowest-index free vertex of the resolved class, bits in order.
#[test]
fn default_placement_takes_lowest_free_vertex_of_the_class() {
    let mut d = grid();
    d["classes"]["slow"] = d["classes"]["fast"].clone();
    d["topology"]["vertices"][1] = json!({"class":"slow"});
    let t = load(&d).unwrap();
    assert_eq!(
        t.default_placement(&[("fast", 2), ("slow", 1), ("fast", 1)])
            .unwrap(),
        vec![vec![0, 2], vec![1], vec![3]]
    );
    let err = t.default_placement(&[("slow", 2)]).unwrap_err();
    assert!(err.contains("§8.3") && err.contains("slow"), "{err}");
}

/// The vertex count limits occupancy in addition to the declared capacity.
#[test]
fn vertex_count_limits_occupancy_below_capacity() {
    let mut d = grid();
    d["classes"]["fast"]["capacity"] = json!("unbounded");
    d["topology"] = json!({"vertices":[{"class":"fast"}],"edges":[]});
    let t = load(&d).unwrap();
    let events = json!({
        "leaves":[{"kind":"classical","start":0,"duration":1}],
        "live_intervals":[
            {"wire":0,"class":"fast","start":0,"end":1},
            {"wire":1,"class":"fast","start":0,"end":1}
        ]
    });
    let err = t.aggregate(&events).unwrap_err();
    assert!(err.contains("vertices"), "{err}");
}

/// Scheme leaves, ancillae included, must themselves satisfy adjacency.
#[test]
fn scheme_leaves_are_checked_against_the_placement() {
    let mut d = grid();
    d["schemes"] = json!([{
        "name":"swap-three-cx",
        "realises":{"op":"swap","controls":{"min":0,"max":0}},
        "ancillae":{"count":1,"state":"clean","class":"fast"},
        "template":[{"gate":"cx","wires":["t","a[0]"]},{"gate":"cx","wires":["a[0]","t2"]}]
    }]);
    let t = load(&d).unwrap();
    let eps = expr::decimal("0").unwrap();
    let leaves = t
        .expand("swap", &["fast", "fast"], 0, 0, eps, None)
        .unwrap();
    // operands t=0, t2=3, ancilla=1: 0-1 and 1-3 are both grid edges.
    t.check_leaves(&leaves, &[0, 3, 1]).unwrap();
    // ancilla=2 with t2=1: 2-1 is a diagonal.
    let err = t.check_leaves(&leaves, &[0, 1, 2]).unwrap_err();
    assert!(err.contains("E_SCHEDULE_INFEASIBLE"), "{err}");
    assert!(t.check_leaves(&leaves, &[0, 3]).is_err());
}

/// Explicit routing with §4.4 swap stays expressible: not a rejection.
#[test]
fn explicitly_routed_swaps_realise_a_non_adjacent_cx() {
    let t = Target::load(&grid_text()).unwrap();
    for pair in [[0, 1], [1, 3]] {
        t.check_placement(&pair).unwrap();
    }
}
