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
/// §8.1: target numbers are exact rationals from their RFC 8785 decimal text.
/// An imported number whose canonical text denotes another rational (binary64
/// rounding, underflow) cannot be carried; fail loudly, never round.
fn preserved(v: &Value, what: &str) -> Result<()> {
    match v {
        Value::Number(_) => {
            let canonical = json::canonical(v)?;
            if expr::decimal(&canonical)? != expr::number(v)? {
                return Err(format!(
                    "import: {what}: {v} cannot be preserved exactly as an RFC 8785 target number (canonical text {canonical}); refusing to round"
                ));
            }
            Ok(())
        }
        Value::Array(a) => a
            .iter()
            .enumerate()
            .try_for_each(|(i, x)| preserved(x, &format!("{what}[{i}]"))),
        Value::Object(o) => o
            .iter()
            .try_for_each(|(k, x)| preserved(x, &format!("{what}.{k}"))),
        _ => Ok(()),
    }
}
fn finish(d: Value, mapping: Value, quantities: Value, source: Value) -> Result<Value> {
    preserved(&d, "target")?;
    let bytes = json::canonical(&d)?;
    let t = Target::load(&bytes)?;
    Ok(
        json!({"canonical":bytes,"target_hash":t.hash,"mapping":mapping,"quantities":quantities,"source":source}),
    )
}
fn reject_unknown(input: &Value, allowed: &[&str], prefix: &str) -> Result<()> {
    for key in object(input)?.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("import: {prefix}{key}: explicitly unsupported field; no silent dropping (ISA §8.3 imports does not define QEC, hierarchy or symbolic-model mappings)"));
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
        let count = expr::nonnegative(n).map_err(|e|format!("import: GateCounts.{field}: concrete nonnegative integral count required; symbolic input needs bindings: {e}"))?;
        if !count.is_integer() {
            return Err(format!(
                "import: GateCounts.{field}: integral count required"
            ));
        }
        let resource = format!("qualtran.{field}");
        insert_resource(&mut d, &resource, field)?;
        map[field] = json!([resource, 1]);
        quantities[&resource] = if count > expr::decimal("9007199254740991")? {
            json!(count.to_integer().to_string())
        } else {
            n.clone()
        };
        coverage[field] =
            json!({"resource":resource,"factor":1,"defaulted":input.get(field).is_none()});
    }
    d["imports"]["qualtran.GateCounts"] = json!({"version":"GateCounts.asdict-7-fields","map":map});
    finish(
        d,
        coverage,
        quantities,
        json!({"format":"qualtran.resource_counting.GateCounts.asdict","input_json":input.to_string(),"base_target_hash":base.hash}),
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
    let v = json::parse(&out)?;
    let canonical = json::canonical(&v)?;
    if expr::decimal(&canonical)? != n {
        return Err(format!(
            "import: {field}: converted duration cannot be preserved exactly as a JCS number"
        ));
    }
    Ok(v)
}
/// One rate: a number, or Azure's {"process","readout"} measurement object.
fn rate_parts(v: &Value, field: &str, allow_object: bool) -> Result<Vec<(String, Value)>> {
    let parts = match v.as_object() {
        Some(o) if allow_object => {
            if o.len() != 2 || !o.contains_key("process") || !o.contains_key("readout") {
                return Err(format!(
                    "import: qubitParams.{field}: object rate has exactly process and readout"
                ));
            }
            vec![
                (format!("azure.{field}.process"), o["process"].clone()),
                (format!("azure.{field}.readout"), o["readout"].clone()),
            ]
        }
        _ => vec![(format!("azure.{field}"), v.clone())],
    };
    for (_, n) in &parts {
        let p = expr::nonnegative(n).map_err(|e| format!("import: qubitParams.{field}: {e}"))?;
        preserved(n, &format!("qubitParams.{field}"))?;
        if p > Rational::from_integer(1.into()) {
            return Err(format!(
                "import: qubitParams.{field}: failure probability above 1"
            ));
        }
    }
    Ok(parts)
}
type TimeRow = (&'static str, Option<&'static str>, &'static [&'static str]);
const GATE_TIMES: [TimeRow; 4] = [
    ("oneQubitMeasurementTime", None, &[]),
    ("oneQubitGateTime", None, &["x", "h", "s", "sdg"]),
    (
        "twoQubitGateTime",
        Some("oneQubitGateTime"),
        &["cx", "cz", "swap"],
    ),
    ("tGateTime", Some("oneQubitGateTime"), &["t", "tdg"]),
];
const MAJORANA_TIMES: [TimeRow; 3] = [
    ("oneQubitMeasurementTime", None, &[]),
    (
        "twoQubitJointMeasurementTime",
        Some("oneQubitMeasurementTime"),
        &[],
    ),
    ("tGateTime", None, &["t", "tdg"]),
];
const GATE_RATES: [TimeRow; 5] = [
    ("oneQubitMeasurementErrorRate", None, &[]),
    ("oneQubitGateErrorRate", None, &["x", "h", "s", "sdg"]),
    (
        "twoQubitGateErrorRate",
        Some("oneQubitGateErrorRate"),
        &["cx", "cz", "swap"],
    ),
    (
        "tGateErrorRate",
        Some("oneQubitGateErrorRate"),
        &["t", "tdg"],
    ),
    ("idleErrorRate", Some("oneQubitMeasurementErrorRate"), &[]),
];
const MAJORANA_RATES: [TimeRow; 4] = [
    ("oneQubitMeasurementErrorRate", None, &[]),
    (
        "twoQubitJointMeasurementErrorRate",
        Some("oneQubitMeasurementErrorRate"),
        &[],
    ),
    ("tGateErrorRate", None, &["t", "tdg"]),
    ("idleErrorRate", Some("oneQubitMeasurementErrorRate"), &[]),
];
/// Legacy Azure estimator qubitParams JSON. Hardware data overlays an explicit
/// base target. North Star §7: quantities that do not align with a declared
/// resource import as new named resources, so a failure probability is a §8.2
/// count resource and is never treated as §8.1 operator-norm epsilon.
pub fn azure(base: &Target, input: &Value) -> Result<Value> {
    if base.document["resources"]["time"]["unit"] != "second"
        && base.document["resources"]["time"]["unit"] != "s"
    {
        return Err("import: base time unit must be second or s".into());
    }
    reject_unknown(input, &["qubitParams"], "")?;
    let q = input
        .get("qubitParams")
        .ok_or("import: qubitParams is required")?;
    let (times, rates): (&[TimeRow], &[TimeRow]) = match q["instructionSet"].as_str() {
        Some("GateBased") => (&GATE_TIMES, &GATE_RATES),
        Some("Majorana") => (&MAJORANA_TIMES, &MAJORANA_RATES),
        _ => {
            return Err("import: qubitParams.instructionSet: GateBased or Majorana required".into())
        }
    };
    let mut fields = vec!["name", "instructionSet"];
    fields.extend(times.iter().chain(rates).map(|row| row.0));
    reject_unknown(q, &fields, "qubitParams.")?;
    let name = q["name"]
        .as_str()
        .ok_or("import: qubitParams.name string required")?;
    let mut d = base.document.clone();
    let mut mapping =
        json!({"name":"source.name (provenance)","instructionSet":q["instructionSet"]});
    let mut map = json!({});
    let mut quantities = json!({});
    if d["imports"].get("azure.qubitParams").is_some() {
        return Err("import: azure.qubitParams already present".into());
    }
    for (field, fallback, ops) in times {
        let source = q
            .get(*field)
            .or_else(|| fallback.and_then(|f| q.get(f)))
            .ok_or_else(|| format!("import: qubitParams.{field} required"))?;
        let value = seconds(source, field)?;
        let resource = format!("azure.{field}.seconds");
        insert_resource(&mut d, &resource, "second")?;
        map[*field] = json!([resource, 1]);
        quantities[&resource] = value.clone();
        for g in d["gates"].as_array_mut().unwrap() {
            if ops.contains(&g["op"].as_str().unwrap()) {
                g["cost"]["time"] = value.clone();
            }
        }
        mapping[*field] = json!({"resource":resource,"unit":"second","gate_time_overlays":ops,"defaulted":q.get(*field).is_none()});
    }
    for (field, fallback, ops) in rates {
        let source = q
            .get(*field)
            .or_else(|| fallback.and_then(|f| q.get(f)))
            .ok_or_else(|| format!("import: qubitParams.{field} required"))?;
        // Gate rates charge per gate, so only a measurement or idle rate may split.
        let parts = rate_parts(source, field, ops.is_empty())?;
        let mut resources = Vec::new();
        for (resource, value) in parts {
            insert_resource(&mut d, &resource, "failure_probability")?;
            let quantity = resource.strip_prefix("azure.").unwrap();
            map[quantity] = json!([resource, 1]);
            quantities[&resource] = value.clone();
            for g in d["gates"].as_array_mut().unwrap() {
                if ops.contains(&g["op"].as_str().unwrap()) {
                    g["cost"][&resource] = value.clone();
                }
            }
            resources.push(resource);
        }
        mapping[*field] = json!({"resources":resources,"unit":"failure_probability","gate_cost_overlays":ops,"defaulted":q.get(*field).is_none()});
    }
    d["imports"]["azure.qubitParams"] = json!({"version":"legacy-qubitParams","map":map});
    finish(
        d,
        mapping,
        quantities,
        json!({"format":"Azure legacy estimator qubitParams","name":name,"input_json":input.to_string(),"base_target_hash":base.hash,"measurement_time_semantics":"imported hardware quantity; ISA §8 has no separate measurement duration field; no operational latency is inferred","failure_probability_semantics":"§8.2 count: the sum of per-gate probabilities over leaf events, a first-order accounting dimension. Not §8.1 epsilon and not a composed failure probability. Measurement and idle rates are quantities only: §8 has no such charged event.","gate_set_semantics":"the base target's gates are retained; Majorana joint measurements are not gates and none is invented"}),
    )
}
