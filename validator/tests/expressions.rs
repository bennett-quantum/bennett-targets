use bennett_targets_validator::{expr::{self,Bindings},json};
use serde_json::json;
#[test]
fn exact_arithmetic_contexts_and_failures() {
 let env=Bindings::from([("eps".into(),expr::decimal("0.001").unwrap()),("k".into(),expr::decimal("1").unwrap())]);
 assert_eq!(expr::eval(&json!({"op":"log2inv_eps","args":[]}),&env,&json!({})).unwrap(),expr::decimal("10").unwrap());
 let e=json::parse(r#"{"op":"+","args":[0.1,0.2]}"#).unwrap();assert_eq!(expr::eval(&e,&env,&json!({})).unwrap(),expr::decimal("0.3").unwrap());
 for e in [json!({"op":"/","args":[1,{"op":"-","args":[{"var":"k"},{"var":"k"}]}]}),json!({"var":"theta"}),json!({"table":"t","index":"k"}),json!({"op":"+","args":[1]}),json!({"var":"k","extra":0})]{assert!(expr::eval(&e,&env,&json!({"t":[1]})).is_err());}
 assert!(expr::eval(&json!({"var":"eps"}),&Bindings::from([("eps".into(),expr::decimal("2").unwrap())]),&json!({})).is_err());
 assert!(expr::cost(&json!(-1),&env,&json!({})).is_err());
 assert_eq!(expr::decimal("123456789012345678901234567890.125").unwrap()*expr::decimal("8").unwrap(),expr::decimal("987654312098765431209876543121").unwrap());
}
#[test]
fn constant_subexpressions_are_checked_at_load() {
 let mut v=json::parse(include_str!("../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json")).unwrap();
 v["gates"][0]["cost"]["time"]=json!({"op":"+","args":[{"var":"k"},{"op":"/","args":[1,0]}]});
 assert!(bennett_targets_validator::Target::load(&json::canonical(&v).unwrap()).is_err());
}
