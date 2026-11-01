use bennett_targets_validator::Target;
#[test]
fn models_are_canonical_and_have_provenance() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../targets");
    for name in [
        "ideal-all-to-all",
        "superconducting-grid-illustrative",
        "superconducting-grid-2d-illustrative",
        "trapped-ion-illustrative",
    ] {
        let path = root.join(format!("{name}.json"));
        let s = std::fs::read_to_string(&path).expect("model must exist");
        Target::load(&s).unwrap();
        let p = std::fs::read_to_string(root.join(format!("{name}.provenance.md"))).unwrap();
        assert!(p.contains("illustrative"));
    }
}

/// Exhaustive unitary check of a leaf list on 3 wires, against Toffoli.
fn simulate(leaves: &[bennett_targets_validator::runtime::Leaf], basis: usize) -> [(f64, f64); 8] {
    let mut s = [(0.0, 0.0); 8];
    s[basis] = (1.0, 0.0);
    let bit = |w: usize| 1usize << (2 - w);
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let mul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
    for l in leaves {
        let mut n = [(0.0, 0.0); 8];
        for (i, a) in s.iter().enumerate() {
            match (l.op.as_str(), l.wires.as_slice()) {
                ("h", [w]) => {
                    let sign = if i & bit(*w) != 0 { -r } else { r };
                    let t = i ^ bit(*w);
                    n[i].0 += a.0 * sign;
                    n[i].1 += a.1 * sign;
                    n[t].0 += a.0 * r;
                    n[t].1 += a.1 * r;
                }
                ("t" | "tdg", [w]) => {
                    let phase = if l.op == "t" { (r, r) } else { (r, -r) };
                    let v = if i & bit(*w) != 0 { mul(*a, phase) } else { *a };
                    n[i].0 += v.0;
                    n[i].1 += v.1;
                }
                ("cx", [c, t]) => {
                    let j = if i & bit(*c) != 0 { i ^ bit(*t) } else { i };
                    n[j].0 += a.0;
                    n[j].1 += a.1;
                }
                ("swap", [x, y]) => {
                    let (bx, by) = (i & bit(*x) != 0, i & bit(*y) != 0);
                    let j = if bx != by { i ^ bit(*x) ^ bit(*y) } else { i };
                    n[j].0 += a.0;
                    n[j].1 += a.1;
                }
                other => panic!("unexpected leaf {other:?}"),
            }
        }
        s = n;
    }
    s
}

/// The grid is bipartite, so a native ccx could never be placed; §8.3 matching
/// takes a gates entry before a scheme, so the model must carry only the scheme.
#[test]
fn grid_2d_ccx_scheme_is_toffoli_and_respects_adjacency() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../targets");
    let s =
        std::fs::read_to_string(root.join("superconducting-grid-2d-illustrative.json")).unwrap();
    let t = Target::load(&s).unwrap();
    assert!(t.document["gates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|g| g["op"] != "ccx"));
    let zero = bennett_targets_validator::expr::decimal("0").unwrap();
    let leaves = t
        .expand("ccx", &["data", "data", "data"], 2, 2, zero, None)
        .unwrap();
    let exact = bennett_targets_validator::expr::decimal("0").unwrap();
    assert!(leaves.iter().all(|l| l.charge.epsilon == exact));
    t.check_leaves(&leaves, &[0, 1, 2]).unwrap();
    t.check_leaves(&leaves, &[0, 1, 5]).unwrap();
    let err = t.check_leaves(&leaves, &[1, 0, 2]).unwrap_err();
    assert!(err.contains("E_SCHEDULE_INFEASIBLE"), "{err}");
    for basis in 0..8usize {
        let out = simulate(&leaves, basis);
        let expected = if basis & 0b110 == 0b110 {
            basis ^ 1
        } else {
            basis
        };
        for (i, a) in out.iter().enumerate() {
            let want = if i == expected { 1.0 } else { 0.0 };
            assert!(
                (a.0 - want).abs() < 1e-9 && a.1.abs() < 1e-9,
                "basis {basis} amplitude {i}: {a:?}"
            );
        }
    }
}
