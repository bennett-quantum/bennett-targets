//! Execute the published scenario data; staged reference outcomes are not passes.
use bennett_targets_validator::{
    expr::{self, Bindings},
    json, Target,
};
use serde_json::{json, Value};
use std::path::Path;
fn read(p: impl AsRef<Path>) -> Value {
    json::parse(&std::fs::read_to_string(p).unwrap()).unwrap()
}
fn rational(v: &Value) -> expr::Rational {
    if let Some(s) = v.as_str() {
        expr::decimal(s).unwrap()
    } else {
        expr::number(v).unwrap()
    }
}
#[test]
fn all_sixteen_published_e_scenarios() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bennett-isa-spec/conformance/cases/E");
    let mut count = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        let case = read(path.join("case.json"));
        let id = case["id"].as_str().unwrap();
        let s = &case["scenario"];
        let expected = &case["expected_outcome"];
        count += 1;
        match id {
            "E-ast-division-by-zero"
            | "E-ast-log2inv"
            | "E-ast-unbound-variable"
            | "E-ast-eps-out-of-range" => {
                let e = read(path.join(s["expression_file"].as_str().unwrap()));
                let mut env = Bindings::new();
                for (k, v) in s["bindings"].as_object().unwrap() {
                    if k != "context" {
                        env.insert(k.clone(), rational(v));
                    }
                }
                let result = expr::eval(&e["expr"], &env, &json!({}));
                if expected.get("value").is_some() {
                    assert_eq!(result.unwrap(), rational(&expected["value"]));
                } else {
                    assert!(result.is_err(), "{id}");
                }
            }
            "E-table-index" => {
                assert!(expr::eval(
                    &s["expression"],
                    &Bindings::from([("k".into(), rational(&s["k"]))]),
                    &s["tables"]
                )
                .is_err());
            }
            "E-epsilon-zero-entry" | "E-mbu-must-be-false" | "E-target-staged-probe" => {
                let input =
                    std::fs::read_to_string(path.join(s["target_file"].as_str().unwrap())).unwrap();
                assert!(Target::load(&input).is_err(), "{id}");
            }
            "E-jcs-target-hash" => {
                let input = std::fs::read_to_string(path.join("target.json")).unwrap();
                assert_eq!(Target::load(&input).unwrap().hash, expected["target_hash"]);
            }
            "E-epsilon-zero-exact" => {
                let input = std::fs::read_to_string(path.join("target.json")).unwrap();
                let target = Target::load(&input).unwrap();
                for eps in s["epsilon_shares"].as_array().unwrap() {
                    let c = target
                        .native("cx", &["fast", "fast"], 1, 1, rational(eps))
                        .unwrap()
                        .unwrap();
                    assert_eq!(c.epsilon, rational(&expected["epsilon"]));
                    assert_eq!(c.cost["time"], rational(&expected["time"]));
                }
            }
            "E-matching-first-entry" => {
                let target =
                    Target::load(&std::fs::read_to_string(path.join("target.json")).unwrap())
                        .unwrap();
                assert!(target
                    .native("cx", &["fast", "fast"], 1, 1, rational(&s["epsilon_share"]))
                    .unwrap()
                    .is_some());
                assert!(target
                    .native(
                        "cx",
                        &["fast", "fast", "fast"],
                        1,
                        2,
                        rational(&s["epsilon_share"])
                    )
                    .unwrap()
                    .is_none());
            }
            "E-scheme-expansion" => {
                let mut d = read(root.join("E-jcs-target-hash/target.json"));
                d["schemes"] = json!([read(path.join("scheme.json"))]);
                let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
                let n = s["k"].as_u64().unwrap() as usize;
                let classes = vec!["fast"; n + 1];
                let leaves = t
                    .expand("mcx", &classes, n, n, expr::decimal("0").unwrap(), None)
                    .unwrap();
                assert_eq!(
                    leaves.len(),
                    expected["leaf_events"].as_u64().unwrap() as usize
                );
                assert_eq!(
                    leaves
                        .iter()
                        .map(|l| l.charge.cost["time"].clone())
                        .sum::<expr::Rational>(),
                    rational(&expected["count_cost"])
                );
            }
            "E-scheme-correctness-negative" => {
                let bits = s["input"].as_array().unwrap();
                let mut value = bits.last().unwrap().as_u64().unwrap();
                for gate in s["template"].as_array().unwrap() {
                    let control = gate[1].as_str().unwrap()[1..].parse::<usize>().unwrap();
                    value ^= bits[control].as_u64().unwrap();
                }
                let ref_value = bits.last().unwrap().as_u64().unwrap()
                    ^ u64::from(bits[..bits.len() - 1].iter().all(|b| b == 1));
                assert_eq!(value, expected["template_target"]);
                assert_eq!(ref_value, expected["reference_target"]);
                assert_ne!(value, ref_value);
            }
            "E-imports" => {
                assert_eq!(
                    rational(&s["quantity"]) * rational(&s["mapping"][1]),
                    rational(&expected["resources"][s["mapping"][0].as_str().unwrap()])
                );
            }
            "E-aggregation" | "E-timing-overlap" => {
                let mut d = read(root.join("E-jcs-target-hash/target.json"));
                d["resources"]["gate_count"] = json!({"kind":"count","unit":"gate"});
                let t = Target::load(&json::canonical(&d).unwrap()).unwrap();
                let events = if id == "E-aggregation" {
                    read(path.join("events.json"))
                } else {
                    let leaves: Vec<_> = s["events"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|e| {
                            let mut e = e.clone();
                            e["kind"] = json!("gate");
                            e
                        })
                        .collect();
                    let intervals: Vec<_> = s["live_intervals"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|l| json!({"wire":l[0],"class":"fast","start":l[1],"end":l[2]}))
                        .collect();
                    json!({"leaves":leaves,"live_intervals":intervals})
                };
                let out = t.aggregate(&events).unwrap();
                for (r, value) in expected.as_object().unwrap() {
                    if value.is_object() {
                        for (c, n) in value.as_object().unwrap() {
                            assert_eq!(out[r][c], rational(n), "{id} {r}");
                        }
                    } else {
                        assert_eq!(out[r]["total"], rational(value), "{id} {r}");
                    }
                }
            }
            _ => panic!("unhandled published E scenario {id}"),
        }
    }
    assert_eq!(count, 16);
}
