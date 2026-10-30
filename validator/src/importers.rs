//! Explicit adapters for concrete exported resource-model data.
use crate::{
    expr::{self, Rational},
    json::{self, Result},
    Target,
};
use num_traits::Zero;
use serde_json::{json, Value};
fn object(v: &Value) -> Result<&serde_json::Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| "import: expected object".into())
}
fn finish(d: Value, mapping: Value, quantities: Value, source: Value) -> Result<Value> {
    let bytes = json::canonical(&d)?;
    let t = Target::load(&bytes)?;
    Ok(
        json!({"canonical":bytes,"target_hash":t.hash,"mapping":mapping,"quantities":quantities,"source":source}),
    )
}
fn reject_unknown(input: &Value, allowed: &[&str], prefix: &str) -> Result<()> {
    for key in object(input)?.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("import: {prefix}{key}: explicitly unsupported field; no silent dropping (ISA §8.3 imports; gap b-targets-gyp for QEC/error models)"));
        }
    }
    Ok(())
}
fn insert_resource(d: &mut Value, name: &str, unit: &str) -> Result<()> {
    if d["resources"].get(name).is_some() {
        return Err(format!("import: resource collision {name}"));
    }
    d["resources"][name] = json!({"kind":"count","unit":unit});
    Ok(())
}
/// GateCounts.asdict(): all seven fields retain distinct count units.
pub fn qualtran(base: &Target, input: &Value) -> Result<Value> {
    let fields = [
        "t",
        "toffoli",
        "cswap",
        "and_bloq",
        "clifford",
        "rotation",
        "measurement",
    ];
    reject_unknown(input, &fields, "GateCounts.")?;
    let mut d = base.document.clone();
    let mut map = json!({});
    let mut quantities = json!({});
    let mut coverage = json!({});
    if d["imports"].get("qualtran.GateCounts").is_some() {
        return Err("import: qualtran.GateCounts already present".into());
    }
    for field in fields {
        let zero = json!(0);
        let n = input.get(field).unwrap_or(&zero);
        expr::integer(n).map_err(|e|format!("import: GateCounts.{field}: concrete nonnegative integral count required; symbolic input needs bindings: {e}"))?;
        let resource = format!("qualtran.{field}");
        insert_resource(&mut d, &resource, field)?;
        map[field] = json!([resource, 1]);
        quantities[&resource] = n.clone();
        coverage[field] =
            json!({"resource":resource,"factor":1,"defaulted":input.get(field).is_none()});
    }
    d["imports"]["qualtran.GateCounts"] = json!({"version":"GateCounts.asdict-7-fields","map":map});
    finish(
        d,
        coverage,
        quantities,
        json!({"format":"qualtran.resource_counting.GateCounts.asdict","input":input,"base_target_hash":base.hash}),
    )
}
fn seconds(v: &Value, field: &str) -> Result<Value> {
    let s = v
        .as_str()
        .ok_or_else(|| format!("import: {field}: duration string required"))?;
    let parts: Vec<_> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("import: {field}: expected '<decimal> <unit>'"));
    }
    let factor = match parts[1] {
        "s" => "1",
        "ms" => "0.001",
        "us" | "µs" | "μs" => "0.000001",
        "ns" => "0.000000001",
        "ps" => "0.000000000001",
        _ => {
            return Err(format!(
                "import: {field}: unsupported duration unit {}",
                parts[1]
            ))
        }
    };
    let n = expr::decimal(parts[0])? * expr::decimal(factor)?;
    if n < Rational::zero() {
        return Err(format!("import: {field}: negative duration"));
    }
    // Both factors are finite decimal, so decimal conversion is exact before JCS.
    let mut denominator = n.denom().clone();
    let ten = num_bigint::BigInt::from(10);
    let mut scale = 0u32;
    let mut power = num_bigint::BigInt::from(1);
    while &power % &denominator != num_bigint::BigInt::zero() {
        power *= &ten;
        scale += 1;
        if scale > 100000 {
            return Err("host limit: duration decimal scale".into());
        }
    }
    denominator = power / denominator;
    let digits = (n.numer() * denominator).to_string();
    let out = if scale == 0 {
        digits
    } else if digits.len() > scale as usize {
        let i = digits.len() - scale as usize;
        format!("{}.{}", &digits[..i], &digits[i..])
    } else {
        format!("0.{}{}", "0".repeat(scale as usize - digits.len()), digits)
    };
    json::parse(&out)
}
/// Legacy Azure estimator qubitParams JSON. Hardware data overlays an explicit
/// base target; stochastic probabilities are never treated as operator-norm eps.
pub fn azure(base: &Target, input: &Value) -> Result<Value> {
    reject_unknown(input, &["qubitParams"], "")?;
    let q = input
        .get("qubitParams")
        .ok_or("import: qubitParams is required")?;
    let fields = [
        "name",
        "instructionSet",
        "oneQubitMeasurementTime",
        "oneQubitGateTime",
        "twoQubitGateTime",
        "tGateTime",
        "oneQubitMeasurementErrorRate",
        "oneQubitGateErrorRate",
        "twoQubitGateErrorRate",
        "tGateErrorRate",
        "idleErrorRate",
    ];
    reject_unknown(q, &fields, "qubitParams.")?;
    if q["instructionSet"] != "GateBased" {
        return Err("import: qubitParams.instructionSet: only explicit GateBased is representable; Majorana joint measurements require ISA channel model b-targets-gyp".into());
    }
    let name = q["name"]
        .as_str()
        .ok_or("import: qubitParams.name string required")?;
    for field in [
        "oneQubitMeasurementErrorRate",
        "oneQubitGateErrorRate",
        "twoQubitGateErrorRate",
        "tGateErrorRate",
        "idleErrorRate",
    ] {
        let fallback = match field {
            "twoQubitGateErrorRate" | "tGateErrorRate" => "oneQubitGateErrorRate",
            "idleErrorRate" => "oneQubitMeasurementErrorRate",
            _ => field,
        };
        let v = q
            .get(field)
            .or_else(|| q.get(fallback))
            .ok_or_else(|| format!("import: qubitParams.{field} required"))?;
        if !expr::nonnegative(v)?.is_zero() {
            return Err(format!("import: qubitParams.{field}: stochastic failure probability cannot be equated to §8.1 approximation epsilon; blocked by b-targets-gyp"));
        }
    }
    let mut d = base.document.clone();
    let mut mapping =
        json!({"name":"source.name (provenance)","instructionSet":"assert GateBased"});
    let mut map = json!({});
    let mut quantities = json!({});
    if d["imports"].get("azure.qubitParams").is_some() {
        return Err("import: azure.qubitParams already present".into());
    }
    for field in [
        "oneQubitMeasurementTime",
        "oneQubitGateTime",
        "twoQubitGateTime",
        "tGateTime",
    ] {
        let source = q
            .get(field)
            .or_else(|| {
                if field == "twoQubitGateTime" || field == "tGateTime" {
                    q.get("oneQubitGateTime")
                } else {
                    None
                }
            })
            .ok_or_else(|| format!("import: qubitParams.{field} required"))?;
        let value = seconds(source, field)?;
        let resource = format!("azure.{field}.seconds");
        insert_resource(&mut d, &resource, "second")?;
        map[field] = json!([resource, 1]);
        quantities[&resource] = value.clone();
        let ops: &[&str] = match field {
            "oneQubitGateTime" => &["x", "h", "s", "sdg"],
            "twoQubitGateTime" => &["cx", "cz", "swap"],
            "tGateTime" => &["t", "tdg"],
            _ => &[],
        };
        for g in d["gates"].as_array_mut().unwrap() {
            if ops.contains(&g["op"].as_str().unwrap()) {
                g["cost"]["time"] = value.clone();
            }
        }
        mapping[field] = json!({"resource":resource,"unit":"second","gate_time_overlays":ops,"defaulted":q.get(field).is_none()});
    }
    for field in [
        "oneQubitMeasurementErrorRate",
        "oneQubitGateErrorRate",
        "twoQubitGateErrorRate",
        "tGateErrorRate",
    ] {
        mapping[field] = json!("assert exact zero; retained in source; nonzero is an error");
    }
    if q.get("idleErrorRate").is_some() {
        mapping["idleErrorRate"] =
            json!("assert exact zero; retained in source; nonzero is an error");
    }
    d["imports"]["azure.qubitParams"] = json!({"version":"legacy-GateBased-qubitParams","map":map});
    finish(
        d,
        mapping,
        quantities,
        json!({"format":"Azure legacy estimator qubitParams","name":name,"input":input,"base_target_hash":base.hash,"measurement_time_semantics":"imported hardware quantity; ISA 0.8 has no separate measurement duration field; no operational latency is inferred"}),
    )
}
