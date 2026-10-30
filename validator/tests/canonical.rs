use bennett_targets_validator::{canon,json,Target};
#[test]
fn suite_e_all_target_bytes_and_hashes() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bennett-isa-spec/conformance/cases/E");
 let mut n=0;
 for d in std::fs::read_dir(root).unwrap(){let p=d.unwrap().path().join("target.json");if !p.exists(){continue;}
  let s=std::fs::read_to_string(&p).unwrap(); let (c,h)=canon(&s).unwrap();assert_eq!(c,s,"{}",p.display());assert_eq!(h,json::hash(s.as_bytes())); n+=1;
  if p.parent().unwrap().ends_with("E-jcs-target-hash"){assert_eq!(h,"71e1973c642abe3ea4ae5b4c7211fd4e2b2b7817894ca958d138c992cc321596");}
 }
 assert_eq!(n,6);
}
#[test]
fn rfc8785_numbers_unicode_and_strings() {
 let s=r#" { "x": [333333333.33333329,1e30,4.50,2e-3,0.000000000000000000000000001,-0.0], "s":"\u000f\n\"\\/", "\ue000":1, "😀":2 } "#;
 let c=canon(s).unwrap().0;
 assert_eq!(c,r#"{"s":"\u000f\n\"\\/","x":[333333333.3333333,1e+30,4.5,0.002,1e-27,0],"😀":2,"":1}"#);
 for bad in [r#"{"x":1,"\u0078":2}"#, "01", "[1,]", "{\"x\":1,}", "NaN",r#""\ud800""#, "1e999"] {assert!(canon(bad).is_err(),"{bad}");}
}
#[test]
fn validate_requires_exact_canonical_bytes() {
 let s=include_str!("../../../bennett-isa-spec/conformance/cases/E/E-jcs-target-hash/target.json");
 assert!(Target::load(s).is_ok());assert!(Target::load(&format!("{s}\n")).unwrap_err().contains("§8.1"));
}
