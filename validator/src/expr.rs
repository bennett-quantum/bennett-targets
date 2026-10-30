//! Exact decimal-rational arithmetic. No floating point cost evaluation.
use crate::json::Result;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde_json::Value;
use std::collections::BTreeMap;
pub type Rational = BigRational;
pub type Bindings = BTreeMap<String, Rational>;
pub fn decimal(s: &str) -> Result<Rational> {
    let (mantissa, exponent) = s.split_once(['e', 'E']).unwrap_or((s, "0"));
    let exponent: i32 = exponent
        .parse()
        .map_err(|_| "host limit: decimal exponent")?;
    if exponent.unsigned_abs() > 100000 {
        return Err("host limit: decimal exponent >100000".into());
    }
    let scale = mantissa.split_once('.').map_or(0, |(_, f)| f.len() as i32) - exponent;
    let digits: BigInt = mantissa
        .replace('.', "")
        .parse()
        .map_err(|_| "§8.1: expected decimal number")?;
    let power = BigInt::from(10).pow(scale.unsigned_abs());
    Ok(if scale >= 0 {
        Rational::new(digits, power)
    } else {
        Rational::from_integer(digits * power)
    })
}
pub fn number(v: &Value) -> Result<Rational> {
    if !v.is_number() {
        return Err("§8.1: numbers only in this context".into());
    }
    decimal(&v.to_string())
}
pub fn nonnegative(v: &Value) -> Result<Rational> {
    let n = number(v)?;
    if n.is_negative() {
        Err("§8.1: costs MUST be nonnegative and finite".into())
    } else {
        Ok(n)
    }
}
pub fn integer(v: &Value) -> Result<usize> {
    let n = nonnegative(v)?;
    if !n.is_integer() {
        return Err("§8.3: count must be an integer".into());
    }
    n.to_integer()
        .to_usize()
        .ok_or_else(|| "host limit: integer exceeds address space".into())
}
pub fn check(v: &Value, vars: &[&str], tables: &Value) -> Result<()> {
    if v.is_number() {
        number(v)?;
        return Ok(());
    }
    let o = v
        .as_object()
        .ok_or("§8.1: cost expressions are a closed JSON AST")?;
    if o.len() == 1 && o.contains_key("var") {
        let name = o["var"]
            .as_str()
            .ok_or("§8.1: variable name must be a string")?;
        if !vars.contains(&name) {
            return Err(format!("§8.1: referencing a variable that the context does not bind is a load-time error: unbound {name}"));
        }
        return Ok(());
    }
    if o.len() == 2 && o.contains_key("table") && o.contains_key("index") {
        let name = o["table"]
            .as_str()
            .ok_or("§8.1: table name must be a string")?;
        if !tables.get(name).is_some_and(Value::is_array) {
            return Err(format!("§8.1: unknown table {name}"));
        }
        match o["index"].as_str() {
            Some("k") if vars.contains(&"k") => return Ok(()),
            Some("eps_decade") if vars.contains(&"eps") => return Err("§8.1: eps_decade indexing undefined in ISA 0.8; blocked by b-targets-w8o (ISA b-isa-spec-f31)".into()),
            _ => return Err("§8.1: table index k or eps_decade must be bound".into())
        }
    }
    if o.len() != 2 || !o.contains_key("op") || !o.contains_key("args") {
        return Err("§8.1: cost expressions are a closed JSON AST".into());
    }
    let op = o["op"].as_str().ok_or("§8.1: op string required")?;
    let arity = match op {
        "+" | "-" | "*" | "/" | "max" | "min" => 2,
        "ceil" => 1,
        "log2inv_eps" if vars.contains(&"eps") => 0,
        _ => return Err("§8.1: unknown or unbound expression operator".into()),
    };
    let args = o["args"].as_array().ok_or("§8.1: args array required")?;
    if args.len() != arity {
        return Err(format!("§8.1: {op} requires {arity} args"));
    }
    for a in args {
        check(a, vars, tables)?;
    }
    Ok(())
}
pub fn uses_eps(v: &Value) -> bool {
    match v {
        Value::Object(o) => {
            o.get("var").is_some_and(|x| x == "eps")
                || o.get("op").is_some_and(|x| x == "log2inv_eps")
                || o.get("index").is_some_and(|x| x == "eps_decade")
                || o.values().any(uses_eps)
        }
        Value::Array(a) => a.iter().any(uses_eps),
        _ => false,
    }
}
pub fn eval(v: &Value, env: &Bindings, tables: &Value) -> Result<Rational> {
    let vars: Vec<_> = env.keys().map(String::as_str).collect();
    check(v, &vars, tables)?;
    fn epsilon(env: &Bindings) -> Result<Rational> {
        let n = env.get("eps").ok_or("§8.1: unbound eps")?;
        if n <= &Rational::zero() || n > &Rational::one() {
            return Err("§8.1: eps outside (0,1] is a load-time error".into());
        }
        Ok(n.clone())
    }
    if v.is_number() {
        return number(v);
    }
    if let Some(name) = v.get("var").and_then(Value::as_str) {
        return if name == "eps" {
            epsilon(env)
        } else {
            env.get(name)
                .cloned()
                .ok_or_else(|| format!("§8.1: unbound {name}"))
        };
    }
    if let Some(name) = v.get("table").and_then(Value::as_str) {
        let k = env.get("k").ok_or("§8.1: unbound k")?;
        if !k.is_integer() || k.is_negative() {
            return Err("§8.1: table index outside the array".into());
        }
        let i = k
            .to_integer()
            .to_usize()
            .ok_or("§8.1: table index outside the array")?;
        return number(
            tables[name]
                .get(i)
                .ok_or("§8.1: table index outside the array is a load-time error")?,
        );
    }
    let op = v["op"].as_str().unwrap();
    let args = v["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| eval(a, env, tables))
        .collect::<Result<Vec<_>>>()?;
    Ok(match op {
        "+" => &args[0] + &args[1],
        "-" => &args[0] - &args[1],
        "*" => &args[0] * &args[1],
        "/" => {
            if args[1].is_zero() {
                return Err("§8.1: division by zero is a load-time error".into());
            }
            &args[0] / &args[1]
        }
        "min" => args[0].clone().min(args[1].clone()),
        "max" => args[0].clone().max(args[1].clone()),
        "ceil" => Rational::from_integer(args[0].ceil().to_integer()),
        "log2inv_eps" => {
            let mut e = epsilon(env)?;
            let mut j = BigInt::zero();
            while e < Rational::one() {
                e *= Rational::from_integer(2.into());
                j += 1;
            }
            Rational::from_integer(j)
        }
        _ => unreachable!(),
    })
}
pub fn cost(v: &Value, env: &Bindings, tables: &Value) -> Result<Rational> {
    let n = eval(v, env, tables)?;
    if n.is_negative() {
        Err("§8.1: costs MUST be nonnegative and finite".into())
    } else {
        Ok(n)
    }
}
