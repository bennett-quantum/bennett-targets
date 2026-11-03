//! ISA 0.9 §8.3 K86 native realisation models (`bennett.native-model/1`).
//!
//! Field set: the accepted K86 text {version,kind,arity,basis,parameter_bindings,
//! control_models,data}. The applied 0.9 text adds operand_map (b-isa-spec-wk0k,
//! open); it is tolerated here only as a verified operand permutation.
use crate::{
    json::Result,
    target::{arr, covers, fields, obj, targets, text},
    Target,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};
use serde_json::Value;
use std::collections::BTreeSet;

pub const VERSION: &str = "bennett.native-model/1";
const UNPUBLISHED: &str = "0.9 publishes no encoding for it (b-targets-1wr)";

/// Gaussian rational a + b i: the exactly decidable fragment of model entries.
type Complex = (BigRational, BigRational);

fn rational_leaf(v: &Value) -> Result<Option<BigRational>> {
    let Some(a) = v.as_array() else {
        return Ok(None);
    };
    let pair = match a.as_slice() {
        [Value::String(n), Value::String(d)] => (n, d),
        _ => return Err("§8.3 K86: a rational constant is a [numerator,positive_denominator] decimal-string pair".into()),
    };
    let canonical_int = |s: &str| {
        let u = s.strip_prefix('-').unwrap_or(s);
        !u.is_empty()
            && u.bytes().all(|b| b.is_ascii_digit())
            && (u == "0" || !u.starts_with('0'))
            && s != "-0"
    };
    if !canonical_int(pair.0) || !canonical_int(pair.1) || pair.1.starts_with('-') {
        return Err(format!(
            "§8.3 K86: rational {pair:?} must be canonical decimal strings with a positive denominator"
        ));
    }
    let n: BigInt = pair.0.parse().unwrap();
    let d: BigInt = pair.1.parse().unwrap();
    if d.is_zero() {
        return Err(format!(
            "§8.3 K86: rational {pair:?} needs a positive denominator"
        ));
    }
    let r = BigRational::new(n.clone(), d.clone());
    if r.numer() != &n || r.denom() != &d {
        return Err(format!(
            "§8.3 K86: rational {pair:?} must be in reduced form"
        ));
    }
    Ok(Some(r))
}

/// Validate an entry expression; return its exact value when it lies in the
/// Gaussian-rational fragment (no variable, pi, sqrt, exp, sin or cos).
fn expression(v: &Value, vars: &BTreeSet<&str>) -> Result<Option<Complex>> {
    if let Some(r) = rational_leaf(v)? {
        return Ok(Some((r, BigRational::zero())));
    }
    let o = obj(v)?;
    if o.len() == 1 && o.contains_key("var") {
        let name = text(&o["var"])?;
        if !vars.contains(name) {
            return Err(format!(
                "§8.3 K86: model variable {name} is not a declared parameter_bindings name"
            ));
        }
        return Ok(None);
    }
    fields(v, &["op", "args"], &[])?;
    let op = text(&o["op"])?;
    let want = match op {
        "pi" | "i" => 0,
        "sqrt" | "exp" | "sin" | "cos" => 1,
        "+" | "-" | "*" | "/" => 2,
        _ => return Err(format!("§8.3 K86: unknown model expression operator {op}")),
    };
    let args = arr(&o["args"])?;
    if args.len() != want {
        return Err(format!("§8.3 K86: {op} takes {want} args"));
    }
    let values = args
        .iter()
        .map(|a| expression(a, vars))
        .collect::<Result<Vec<_>>>()?;
    if values.iter().any(Option::is_none) {
        return Ok(None);
    }
    let x: Vec<Complex> = values.into_iter().flatten().collect();
    let zero = BigRational::zero;
    Ok(match op {
        "i" => Some((zero(), BigRational::one())),
        "+" => Some((&x[0].0 + &x[1].0, &x[0].1 + &x[1].1)),
        "-" => Some((&x[0].0 - &x[1].0, &x[0].1 - &x[1].1)),
        "*" => Some(mul(&x[0], &x[1])),
        "/" => {
            let n = &x[1].0 * &x[1].0 + &x[1].1 * &x[1].1;
            if n.is_zero() {
                return Err("§8.3 K86: division by zero in a model expression".into());
            }
            let p = mul(&x[0], &(x[1].0.clone(), -x[1].1.clone()));
            Some((p.0 / &n, p.1 / &n))
        }
        _ => None, // pi, sqrt, exp, sin, cos: exact but outside this decidable fragment
    })
}
fn mul(a: &Complex, b: &Complex) -> Complex {
    (&a.0 * &b.0 - &a.1 * &b.1, &a.0 * &b.1 + &a.1 * &b.0)
}

/// The §4.4 base unitary that controlled_U wraps, when it is Gaussian-rational.
/// None: the ideal (h, t, tdg, rx, ry, rz) is not a constant Gaussian-rational matrix.
fn ideal(op: &str) -> Option<Vec<Vec<Complex>>> {
    let n = |a: i64, b: i64| {
        (
            BigRational::from_integer(a.into()),
            BigRational::from_integer(b.into()),
        )
    };
    let diag = |d: Vec<Complex>| {
        (0..d.len())
            .map(|r| {
                (0..d.len())
                    .map(|c| if r == c { d[r].clone() } else { n(0, 0) })
                    .collect()
            })
            .collect()
    };
    Some(match op {
        "x" | "cx" | "ccx" | "mcx" => vec![vec![n(0, 0), n(1, 0)], vec![n(1, 0), n(0, 0)]],
        "s" => diag(vec![n(1, 0), n(0, 1)]),
        "sdg" => diag(vec![n(1, 0), n(0, -1)]),
        "cz" => diag(vec![n(1, 0), n(1, 0), n(1, 0), n(-1, 0)]),
        "swap" => (0..4)
            .map(|r| {
                (0..4)
                    .map(|c| {
                        if [0, 2, 1, 3][r] == c {
                            n(1, 0)
                        } else {
                            n(0, 0)
                        }
                    })
                    .collect()
            })
            .collect(),
        _ => return None,
    })
}

impl Target {
    /// §8.3 K86: check one gates entry's realisation_model.
    pub(crate) fn realisation_model(&self, gate: &Value) -> Result<()> {
        let m = &gate["realisation_model"];
        let version = obj(m)?
            .get("version")
            .map(text)
            .transpose()?
            .ok_or("§8.3 K86: realisation_model requires version")?;
        if version != VERSION {
            return Err(format!(
                "§8.3 K86: unsupported realisation_model version {version}; unsupported-model reporting, no operation is guessed"
            ));
        }
        fields(
            m,
            &[
                "version",
                "kind",
                "arity",
                "basis",
                "parameter_bindings",
                "control_models",
                "data",
            ],
            &["operand_map"],
        )?;
        if m["kind"] != "unitary" {
            return Err("§8.3 K86: the native model kind is unitary".into());
        }
        let op = text(&gate["op"])?;
        if m["control_models"]
            != serde_json::json!({"kind":"controlled_U","controls":"c","disabled":"identity"})
        {
            return Err(format!(
                "§8.3 K86: control_models must be the controlled_U rule; explicit control models are named but {UNPUBLISHED}"
            ));
        }
        // controlled_U: arity/basis describe the base matrix on the §4.4 targets.
        let arity = crate::expr::integer(&m["arity"])?;
        if arity != targets(op) {
            return Err(format!(
                "§8.3 K86: controlled_U base arity of {op} is {}, not {arity}",
                targets(op)
            ));
        }
        let basis = arr(&m["basis"])?
            .iter()
            .map(text)
            .collect::<Result<BTreeSet<_>>>()?;
        if basis.len() != arity || arr(&m["basis"])?.len() != arity {
            return Err("§8.3 K86: basis names arity distinct operand bits".into());
        }
        let exact = self.constant_zero(&gate["epsilon"])?;
        let mut allowed = vec!["k", "c", "eps"];
        if gate.get("params").is_some() {
            allowed.push("theta");
        }
        let mut vars = BTreeSet::new();
        for (name, binding) in obj(&m["parameter_bindings"])? {
            let binding = text(binding)?;
            if !allowed.contains(&binding) {
                return Err(format!(
                    "§8.3 K86: parameter_bindings {name} -> {binding} names no context binding"
                ));
            }
            if binding == "eps" && exact {
                return Err(
                    "§8.3 K86/§8.1: eps is bound only where evaluated epsilon is positive".into(),
                );
            }
            vars.insert(name.as_str());
        }
        fields(&m["data"], &["matrix"], &[]).map_err(|e| {
            format!("§8.3 K86: data carries the phase-bearing matrix; other representations: {UNPUBLISHED}: {e}")
        })?;
        let dim = 1usize << arity;
        let rows = arr(&m["data"]["matrix"])?;
        let mut decided: Option<Vec<Vec<Complex>>> = Some(Vec::new());
        if rows.len() != dim {
            return Err(format!("§8.3 K86: matrix dimension must be {dim}x{dim}"));
        }
        for row in rows {
            let row = arr(row)?;
            if row.len() != dim {
                return Err(format!("§8.3 K86: matrix dimension must be {dim}x{dim}"));
            }
            let mut out = Vec::new();
            for entry in row {
                let parts = arr(entry)?;
                if parts.len() != 2 {
                    return Err("§8.3 K86: a matrix entry is [real,imaginary]".into());
                }
                let (re, im) = (expression(&parts[0], &vars)?, expression(&parts[1], &vars)?);
                if let (Some(re), Some(im)) = (re, im) {
                    let i_im = mul(&(BigRational::zero(), BigRational::one()), &im);
                    out.push((re.0 + i_im.0, re.1 + i_im.1));
                }
            }
            match decided.as_mut() {
                Some(d) if out.len() == dim => d.push(out),
                _ => decided = None,
            }
        }
        if let Some(map) = m.get("operand_map") {
            let map = arr(map)?
                .iter()
                .map(crate::expr::integer)
                .collect::<Result<Vec<_>>>()?;
            let mut sorted = map.clone();
            sorted.sort_unstable();
            let c = map.len().checked_sub(arity);
            let Some(c) = c.filter(|_| sorted.iter().copied().eq(0..map.len())) else {
                return Err(format!(
                    "§8.3 K86: operand_map {map:?} must be a permutation of the {arity} base bits plus the resolved controls"
                ));
            };
            // c is total controls, so it must hold the §4.4 own controls (cx 1, ccx 2).
            if !covers(&gate["controls"], c)? || c < own(op) {
                return Err(format!(
                    "§8.3 K86: operand_map {map:?} resolves c={c}, which this {op} entry does not admit"
                ));
            }
        }
        // §8.3 K86: a supplied model MUST agree with the exact §4.4 operation in
        // every zero-epsilon context. Decided exactly here when the matrix is
        // Gaussian-rational; otherwise it remains a checker obligation.
        if let (true, Some(u)) = (exact, decided) {
            if ideal(op).is_none_or(|v| v != u) {
                return Err(format!(
                    "§8.3 K86: realisation_model contradicts the exact zero-epsilon §4.4 {op}"
                ));
            }
        }
        Ok(())
    }
}
fn own(op: &str) -> usize {
    match op {
        "cx" => 1,
        "ccx" => 2,
        _ => 0,
    }
}
