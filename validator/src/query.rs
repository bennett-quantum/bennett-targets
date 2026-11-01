//! CLI queries: a black-box surface for differential §8 outcome comparison.
use bennett_targets_validator::{
    expr::{self, Bindings},
    Target,
};
use serde_json::{json, Value};

fn only(q: &Value, keys: &[&str]) -> Result<(), String> {
    let o = q.as_object().ok_or("query: expected object")?;
    for k in keys {
        if !o.contains_key(*k) {
            return Err(format!("query: missing {k}"));
        }
    }
    match o.keys().find(|k| !keys.contains(&k.as_str())) {
        Some(k) => Err(format!("query: unknown field {k}")),
        None => Ok(()),
    }
}

/// {"expr":<§8.1 expr>,"bindings":{"eps":"0.1",...}} -> an exact reduced rational.
pub fn eval(t: &Target, q: &Value) -> Result<Value, String> {
    only(q, &["expr", "bindings"])?;
    let mut env = Bindings::new();
    for (k, v) in q["bindings"].as_object().ok_or("query: bindings object")? {
        let text = v
            .as_str()
            .ok_or("query: binding values are decimal strings")?;
        env.insert(k.clone(), expr::decimal(text)?);
    }
    let value = expr::eval(&q["expr"], &env, &t.document["tables"])?;
    Ok(json!({"value":value.to_string()}))
}

/// {"vertices":[u,v,...]} -> realisable, or the §17 item 15 infeasibility error.
pub fn adjacency(t: &Target, q: &Value) -> Result<Value, String> {
    only(q, &["vertices"])?;
    let vertices = q["vertices"]
        .as_array()
        .ok_or("query: vertices array")?
        .iter()
        .map(expr::integer)
        .collect::<Result<Vec<_>, _>>()?;
    t.check_placement(&vertices)?;
    Ok(json!({"realisable":true,"vertices":vertices}))
}
