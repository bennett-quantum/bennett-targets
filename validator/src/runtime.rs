//! Context-dependent target checks, expansion and scheduled-event accounting.
use crate::{
    expr::{self, Bindings, Rational},
    json::{self, Result},
    target::{covers, Charge},
    Target,
};
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone)]
pub struct Leaf {
    pub op: String,
    pub wires: Vec<usize>,
    pub charge: Charge,
    pub params: Vec<Rational>,
}
pub type Resources = BTreeMap<String, BTreeMap<String, Rational>>;
fn index(n: Rational) -> Result<usize> {
    if !n.is_integer() || n.is_negative() {
        return Err("§8.3: wire indices and ancilla counts must be nonnegative integers".into());
    }
    n.to_integer()
        .to_usize()
        .ok_or_else(|| "host limit: count exceeds address space".into())
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    v.as_array()
        .ok_or_else(|| "§8.2/§8.3: expected array".into())
}
fn object(v: &Value) -> Result<&serde_json::Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| "§8.2/§8.3: expected object".into())
}
fn own(op: &str, n: usize) -> Result<usize> {
    Ok(match op {
        "cx" => 1,
        "ccx" => 2,
        "mcx" => n.checked_sub(1).ok_or("§8.3: mcx needs target")?,
        _ => 0,
    })
}
impl Target {
    pub fn expand(
        &self,
        op: &str,
        classes: &[&str],
        k: usize,
        c: usize,
        eps: Rational,
        theta: Option<Rational>,
    ) -> Result<Vec<Leaf>> {
        for cls in classes {
            self.class(cls)?;
        }
        let inherited = c.checked_sub(k).ok_or("§8.3: c must include k")?;
        let limit = &self.document["capabilities"]["max_predicate_depth"];
        if limit != "unbounded" && inherited > expr::integer(limit)? {
            return Err("§8.3: max_predicate_depth exceeded".into());
        }
        if let Some(charge) =
            self.native_parameters(op, classes, k, c, eps.clone(), theta.is_some())?
        {
            return Ok(vec![Leaf {
                op: op.into(),
                wires: (0..classes.len()).collect(),
                charge,
                params: theta.into_iter().collect(),
            }]);
        }
        let tables = &self.document["tables"];
        for s in array(&self.document["schemes"])? {
            if s["realises"]["op"] != op || !covers(&s["realises"]["controls"], c)? {
                continue;
            }
            if s["realises"].get("params").is_some() != theta.is_some() {
                continue;
            }
            let mut env = Bindings::from([
                ("k".into(), Rational::from_integer(k.into())),
                ("c".into(), Rational::from_integer(c.into())),
            ]);
            if let Some(t) = theta.clone() {
                env.insert("theta".into(), t);
            }
            let count = index(expr::eval(&s["ancillae"]["count"], &env, tables)?)?;
            let targets = if op == "swap" || op == "cz" { 2 } else { 1 };
            let target_start = c;
            let required = c + targets;
            if classes.len() != required {
                return Err("§8.3: operand classes must cover controls and targets".into());
            }
            let anc_class = s["ancillae"]["class"].as_str().unwrap();
            let mut leaves = Vec::new();
            for g in array(&s["template"])? {
                let (from, to) = if let Some(r) = g.get("repeat") {
                    let from = expr::eval(&r["from"], &env, tables)?;
                    let to = expr::eval(&r["to"], &env, tables)?;
                    if !from.is_integer() || !to.is_integer() {
                        return Err("§8.3: repeat bounds must be integers".into());
                    }
                    (from.to_integer(), to.to_integer())
                } else {
                    (0.into(), 1.into())
                };
                let mut i = from;
                while i < to {
                    let mut env = env.clone();
                    if g.get("repeat").is_some() {
                        env.insert("i".into(), Rational::from_integer(i.clone()));
                    }
                    let mut wires = Vec::new();
                    let mut cls = Vec::new();
                    for w in array(&g["wires"])? {
                        let w = w.as_str().ok_or("§8.3: wire string")?;
                        let n = match w {
                            "t" => target_start,
                            "t2" if targets == 2 => target_start + 1,
                            "t2" => return Err("§8.3: t2 only for swap/cz".into()),
                            _ => {
                                let e = json::parse(&w[2..w.len() - 1]).unwrap_or_else(
                                    |_| serde_json::json!({"var":&w[2..w.len()-1]}),
                                );
                                let n = index(expr::eval(&e, &env, tables)?)?;
                                if w.starts_with('c') {
                                    if n >= c {
                                        return Err(
                                            "§8.3: control subscript outside controls".into()
                                        );
                                    }
                                    n
                                } else {
                                    if n >= count {
                                        return Err("§8.3: ancilla subscript outside count".into());
                                    }
                                    classes.len() + n
                                }
                            }
                        };
                        if wires.contains(&n) {
                            return Err("§8.3: template aliases gate operands".into());
                        }
                        wires.push(n);
                        cls.push(if n < classes.len() {
                            classes[n]
                        } else {
                            anc_class
                        });
                    }
                    let op = g["gate"].as_str().unwrap();
                    let k = own(op, wires.len())?;
                    let charge=self.native_parameters(op,&cls,k,k,eps.clone(),g.get("params").is_some())?.ok_or("§8.3: Every template gate MUST itself match a gates entry (schemes do not nest)")?;
                    let params = g
                        .get("params")
                        .map(|p| {
                            array(p)?
                                .iter()
                                .map(|v| expr::eval(v, &env, tables))
                                .collect::<Result<Vec<_>>>()
                        })
                        .transpose()?
                        .unwrap_or_default();
                    leaves.push(Leaf {
                        op: op.into(),
                        wires,
                        charge,
                        params,
                    });
                    i += 1;
                }
            }
            return Ok(leaves);
        }
        Err("§8.3: no gates entry or scheme covers operation; load-time error".into())
    }
    pub fn import_quantities(
        &self,
        model: &str,
        quantities: &Bindings,
    ) -> Result<BTreeMap<String, Rational>> {
        let map = self.document["imports"]
            .get(model)
            .ok_or("§8.3: unknown imported model")?
            .get("map")
            .ok_or("§8.3: imports map missing")?;
        let mut out = BTreeMap::new();
        for (q, n) in quantities {
            if n.is_negative() {
                return Err("§8.1: nonnegative quantities required".into());
            }
            let mapping = map
                .get(q)
                .ok_or_else(|| format!("§8.3: unmapped imported quantity {q}"))?;
            let r = mapping[0].as_str().ok_or("§8.3: resource name")?;
            *out.entry(r.into()).or_insert_with(Rational::zero) += n * expr::number(&mapping[1])?;
        }
        Ok(out)
    }
    pub fn aggregate(&self, input: &Value) -> Result<Resources> {
        let events = array(&input["leaves"])?;
        let intervals = array(&input["live_intervals"])?;
        let mut points = BTreeSet::new();
        let mut makespan = Rational::zero();
        let mut event_data = Vec::new();
        let mut interval_data = Vec::new();
        let mut totals = Resources::new();
        for (r, def) in object(&self.document["resources"])? {
            let mut map = BTreeMap::new();
            if def["kind"] == "count" || def["kind"] == "time" {
                map.insert("total".into(), Rational::zero());
            } else {
                for c in object(&self.document["classes"])?.keys() {
                    map.insert(c.clone(), Rational::zero());
                }
            }
            totals.insert(r.clone(), map);
        }
        for e in events {
            let start = expr::nonnegative(&e["start"])?;
            let duration = expr::nonnegative(&e["duration"])?;
            let end = &start + &duration;
            let kind = e["kind"].as_str().ok_or("§8.2: leaf event kind required")?;
            if !["gate", "classical", "effect"].contains(&kind) {
                return Err("§8.2: every leaf exactly once; unknown/non-leaf event".into());
            }
            if (kind == "classical" || kind == "effect")
                && duration != expr::number(&self.document["capabilities"]["classical_step_time"])?
            {
                return Err(
                    "§8.2: Classical instructions and effects take classical_step_time".into(),
                );
            }
            let wires = e
                .get("wires")
                .map(|w| {
                    array(w)?
                        .iter()
                        .map(expr::integer)
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            if wires.iter().collect::<BTreeSet<_>>().len() != wires.len() {
                return Err("§8.2: duplicate event wires".into());
            }
            if let Some(cost) = e.get("cost") {
                for (r, n) in object(cost)? {
                    let def = self.document["resources"]
                        .get(r)
                        .ok_or("§8.2: undeclared resource")?;
                    let n = expr::nonnegative(n)?;
                    if def["kind"] == "count" {
                        *totals.get_mut(r).unwrap().get_mut("total").unwrap() += n;
                    }
                }
            }
            makespan = makespan.max(end.clone());
            points.insert(start.clone());
            points.insert(end.clone());
            event_data.push((start, end, wires));
        }
        for l in intervals {
            let wire = expr::integer(&l["wire"])?;
            let cls = l["class"].as_str().ok_or("§8.2: live interval class")?;
            self.class(cls)?;
            let start = expr::nonnegative(&l["start"])?;
            let end = expr::nonnegative(&l["end"])?;
            if end < start {
                return Err("§8.2: live interval ends before start".into());
            }
            if end > makespan {
                return Err("§8.2: wire live interval exceeds scheduled makespan".into());
            }
            let borrowed = l
                .get("borrowed")
                .map(|v| v.as_bool().ok_or("§8.2: borrowed bool"))
                .transpose()?
                .unwrap_or(false);
            points.insert(start.clone());
            points.insert(end.clone());
            interval_data.push((wire, cls, start, end, borrowed));
        }
        for (start, end, wires) in &event_data {
            for w in wires {
                if !interval_data
                    .iter()
                    .any(|(n, _, a, b, _)| n == w && a <= start && b >= end)
                {
                    return Err("§8.2: gate wires require covering live intervals".into());
                }
            }
        }
        let points: Vec<_> = points.into_iter().collect();
        for pair in points.windows(2) {
            let t = &pair[0];
            let dt = &pair[1] - t;
            let active: Vec<_> = event_data
                .iter()
                .filter(|(s, e, _)| s <= t && t < e)
                .collect();
            let mut occupied = BTreeSet::new();
            let mut gate_events = 0;
            for (_, _, ws) in active {
                if !ws.is_empty() {
                    gate_events += 1;
                }
                for w in ws {
                    if !occupied.insert(w) {
                        return Err("§8.2: overlapping gates occupy the same wires".into());
                    }
                }
            }
            if self.document["capabilities"]["max_parallel"] != "unbounded"
                && gate_events > expr::integer(&self.document["capabilities"]["max_parallel"])?
            {
                return Err("§8.2: max_parallel exceeded".into());
            }
            let mut live: BTreeMap<usize, Vec<(&str, bool)>> = BTreeMap::new();
            for (wire, cls, s, e, borrowed) in &interval_data {
                if s <= t && t < e {
                    live.entry(*wire).or_default().push((cls, *borrowed));
                }
            }
            let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
            let mut rates: BTreeMap<&str, Rational> = BTreeMap::new();
            for entries in live.values() {
                let borrows: Vec<_> = entries.iter().filter(|(_, b)| *b).collect();
                let owners: Vec<_> = entries.iter().filter(|(_, b)| !*b).collect();
                if borrows.len() > 1 || owners.len() > 1 {
                    return Err("§8.2: duplicate owner or borrower interval".into());
                }
                let &(cls, borrowed) = borrows
                    .first()
                    .or_else(|| owners.first())
                    .ok_or("§8.2: absent wire ownership")?;
                *counts.entry(cls).or_default() += 1;
                let rate = if *borrowed {
                    expr::number(&self.document["classes"][cls]["dirty_rate"])?
                } else {
                    Rational::one()
                };
                *rates.entry(cls).or_insert_with(Rational::zero) += rate;
            }
            for (cls, n) in &counts {
                let cap = &self.document["classes"][*cls]["capacity"];
                if cap != "unbounded" && *n > expr::integer(cap)? {
                    return Err(format!(
                        "§8.4: capacity constraint is peak live qubits per class: {cls}"
                    ));
                }
            }
            for (r, def) in object(&self.document["resources"])? {
                for (cls, n) in &counts {
                    if def["kind"] == "peak" {
                        let peak = totals.get_mut(r).unwrap().get_mut(*cls).unwrap();
                        *peak = peak.clone().max(Rational::from_integer((*n).into()));
                    }
                    if def["kind"] == "occupancy" {
                        let rate = def
                            .get("rate")
                            .and_then(|r| r.get(*cls))
                            .map(expr::number)
                            .transpose()?
                            .unwrap_or_else(Rational::one);
                        *totals.get_mut(r).unwrap().get_mut(*cls).unwrap() +=
                            &rates[cls] * rate * &dt;
                    }
                }
            }
        }
        for (r, def) in object(&self.document["resources"])? {
            if def["kind"] == "time" {
                totals
                    .get_mut(r)
                    .unwrap()
                    .insert("total".into(), makespan.clone());
            }
        }
        Ok(totals)
    }
}
