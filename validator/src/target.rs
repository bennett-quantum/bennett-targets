use crate::{
    expr::{self, Bindings, Rational},
    json::{self, Result},
};
use num_traits::Zero;
use serde_json::Value;
use std::collections::BTreeMap;
fn obj(v: &Value) -> Result<&serde_json::Map<String, Value>> {
    v.as_object().ok_or_else(|| "§8.3: expected object".into())
}
fn arr(v: &Value) -> Result<&Vec<Value>> {
    v.as_array().ok_or_else(|| "§8.3: expected array".into())
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| "§8.3: expected string".into())
}
fn fields(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    let o = obj(v)?;
    for k in required {
        if !o.contains_key(*k) {
            return Err(format!("§8.3 schema: missing {k}"));
        }
    }
    for k in o.keys() {
        if !required.contains(&k.as_str()) && !optional.contains(&k.as_str()) {
            return Err(format!("§8.3 schema: unknown field {k}"));
        }
    }
    Ok(())
}
fn bound(v: &Value) -> Result<Option<usize>> {
    if v == "unbounded" {
        Ok(None)
    } else {
        expr::integer(v).map(Some)
    }
}
fn controls(v: &Value) -> Result<()> {
    fields(v, &["min", "max"], &[])?;
    let lo = expr::integer(&v["min"])?;
    if bound(&v["max"])?.is_some_and(|hi| hi < lo) {
        return Err("§8.3: inclusive controls range min exceeds max".into());
    }
    Ok(())
}
fn params(v: &Value) -> Result<bool> {
    if v.is_null() {
        return Ok(false);
    }
    if arr(v)? != &vec![Value::String("theta".into())] {
        return Err("§8.3: params must be [theta]".into());
    }
    Ok(true)
}
fn boolean(v: &Value) -> Result<()> {
    if v.is_boolean() {
        Ok(())
    } else {
        Err("§8.3: expected bool".into())
    }
}
#[derive(Clone, Debug)]
pub struct Target {
    pub document: Value,
    pub canonical: String,
    pub hash: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Charge {
    pub epsilon: Rational,
    pub cost: BTreeMap<String, Rational>,
}
impl Target {
    pub fn load(s: &str) -> Result<Self> {
        let document = json::parse(s)?;
        let canonical = json::canonical(&document)?;
        if canonical != s {
            return Err(
                "§8.1: A target is a JSON document in RFC 8785 canonical form (no trailing LF)"
                    .into(),
            );
        }
        let t = Self {
            hash: json::hash(s.as_bytes()),
            document,
            canonical,
        };
        t.schema()?;
        Ok(t)
    }
    pub fn schema(&self) -> Result<()> {
        let d = &self.document;
        fields(
            d,
            &[
                "resources",
                "ptr_widths_supported",
                "classes",
                "gates",
                "schemes",
                "qram",
                "capabilities",
                "objective",
                "tables",
                "imports",
            ],
            &[],
        )?;
        let classes = obj(&d["classes"])?;
        if classes.is_empty() {
            return Err("§8.3: classes must not be empty".into());
        }
        for c in classes.values() {
            fields(c, &["qubit_second", "dirty_rate", "capacity"], &[])?;
            expr::nonnegative(&c["qubit_second"])?;
            expr::nonnegative(&c["dirty_rate"])?;
            bound(&c["capacity"])?;
        }
        let widths = arr(&d["ptr_widths_supported"])?;
        if widths.is_empty() {
            return Err("§8.3: ptr_widths_supported must not be empty".into());
        }
        for (i, w) in widths.iter().enumerate() {
            let w = expr::integer(w)?;
            if ![32, 64].contains(&w) || widths[..i].contains(&widths[i]) {
                return Err("§8.3: unique pointer widths 32 or 64 required".into());
            }
        }
        let resources = obj(&d["resources"])?;
        for (name, kind) in [
            ("qubit_second", "occupancy"),
            ("time", "time"),
            ("peak_qubits", "peak"),
        ] {
            if d["resources"][name]["kind"] != kind {
                return Err(format!("§8.2: Mandatory {name} ({kind})"));
            }
        }
        for r in resources.values() {
            fields(r, &["kind", "unit"], &["rate"])?;
            text(&r["unit"])?;
            if !["count", "time", "occupancy", "peak"].contains(&text(&r["kind"])?) {
                return Err("§8.2: invalid resource kind".into());
            }
            if let Some(rate) = r.get("rate") {
                for (c, n) in obj(rate)? {
                    self.class(c)?;
                    expr::nonnegative(n)?;
                }
            }
        }
        for values in obj(&d["tables"])?.values() {
            for n in arr(values)? {
                expr::number(n)?;
            }
        }
        let cap = &d["capabilities"];
        fields(
            cap,
            &[
                "measurement",
                "mbu",
                "max_parallel",
                "max_predicate_depth",
                "classical_step_time",
            ],
            &[],
        )?;
        if cap["mbu"] != false {
            return Err("§8.3 / §6.4: target capability mbu is reserved and MUST be false".into());
        }
        for key in ["max_parallel", "max_predicate_depth"] {
            bound(&cap[key])?;
        }
        expr::nonnegative(&cap["classical_step_time"])?;
        fields(
            &cap["measurement"],
            &["mid_circuit", "feedforward_latency"],
            &[],
        )?;
        boolean(&cap["measurement"]["mid_circuit"])?;
        expr::nonnegative(&cap["measurement"]["feedforward_latency"])?;
        for g in arr(&d["gates"])? {
            fields(
                g,
                &["op", "classes", "controls", "epsilon", "cost"],
                &["params"],
            )?;
            self.op(&g["op"])?;
            controls(&g["controls"])?;
            params(&g["params"])?;
            if arr(&g["classes"])?.is_empty() {
                return Err("§8.3: class pattern cannot be empty".into());
            }
            for c in arr(&g["classes"])? {
                if c != "*" {
                    self.class(text(c)?)?;
                }
            }
            self.expression(&g["epsilon"], &["k", "c", "eps"], true)?;
            self.costs(&g["cost"], &["k", "c", "eps"])?;
            if !expr::uses_eps(&g["epsilon"])
                && self.constant_zero(&g["epsilon"])?
                && expr::uses_eps(&g["cost"])
            {
                return Err(
                    "§8.1: an entry with epsilon 0 MUST NOT reference eps or log2inv_eps".into(),
                );
            }
        }
        let mut names = std::collections::BTreeSet::new();
        for s in arr(&d["schemes"])? {
            fields(s, &["name", "realises", "ancillae", "template"], &[])?;
            let name = text(&s["name"])?;
            if !names.insert(name) {
                return Err("§8.3: duplicate scheme name".into());
            }
            let r = &s["realises"];
            fields(r, &["op", "controls"], &["params"])?;
            self.op(&r["op"])?;
            controls(&r["controls"])?;
            let param = params(&r["params"])?;
            let vars = if param {
                vec!["k", "c", "theta"]
            } else {
                vec!["k", "c"]
            };
            let a = &s["ancillae"];
            fields(a, &["count", "state", "class"], &[])?;
            self.class(text(&a["class"])?)?;
            if !["clean", "dirty"].contains(&text(&a["state"])?) {
                return Err("§8.3: ancilla state clean or dirty required".into());
            }
            self.expression(&a["count"], &vars, true)?;
            for g in arr(&s["template"])? {
                fields(g, &["gate", "wires"], &["params", "repeat"])?;
                self.op(&g["gate"])?;
                if !arr(&d["gates"])?.iter().any(|n| n["op"] == g["gate"]) {
                    return Err("§8.3: Every template gate MUST itself match a gates entry (schemes do not nest)".into());
                }
                let mut gv = vars.clone();
                if let Some(r) = g.get("repeat") {
                    fields(r, &["var", "from", "to"], &[])?;
                    if r["var"] != "i" {
                        return Err("§8.3: repeat binds i".into());
                    }
                    self.expression(&r["from"], &vars, false)?;
                    self.expression(&r["to"], &vars, false)?;
                    gv.push("i");
                }
                for w in arr(&g["wires"])? {
                    self.wire(text(w)?, &gv)?;
                }
                if let Some(p) = g.get("params") {
                    for e in arr(p)? {
                        let mut pv = Vec::new();
                        if param {
                            pv.push("theta");
                        }
                        if gv.contains(&"i") {
                            pv.push("i");
                        }
                        self.expression(e, &pv, false)?;
                    }
                }
            }
        }
        if !d["qram"].is_null() {
            fields(&d["qram"], &["load", "store", "exch"], &[])?;
            for op in ["load", "store", "exch"] {
                fields(&d["qram"][op], &["cost"], &[])?;
                self.costs(&d["qram"][op]["cost"], &["a", "w", "c"])?;
            }
        }
        let objective = obj(&d["objective"])?;
        if objective.len() != 1 {
            return Err("§8.3: objective weighted or lexicographic".into());
        }
        if let Some(w) = objective.get("weighted") {
            for (r, n) in obj(w)? {
                self.resource(r)?;
                expr::nonnegative(n)?;
            }
        } else if let Some(l) = objective.get("lexicographic") {
            for r in arr(l)? {
                self.resource(text(r)?)?;
            }
        } else {
            return Err("§8.3: unknown objective".into());
        }
        for model in obj(&d["imports"])?.values() {
            fields(model, &["version", "map"], &[])?;
            text(&model["version"])?;
            for mapping in obj(&model["map"])?.values() {
                let m = arr(mapping)?;
                if m.len() != 2 {
                    return Err("§8.3: imports map requires [resource,factor]".into());
                }
                self.resource(text(&m[0])?)?;
                expr::nonnegative(&m[1])?;
            }
        }
        Ok(())
    }
    fn constant_zero(&self, v: &Value) -> Result<bool> {
        if contains_variable(v) {
            return Ok(false);
        }
        Ok(expr::eval(v, &Bindings::new(), &self.document["tables"])?.is_zero())
    }
    pub fn class(&self, c: &str) -> Result<()> {
        if self.document["classes"].get(c).is_some() {
            Ok(())
        } else {
            Err(format!("§8.3: unknown class {c}"))
        }
    }
    fn resource(&self, r: &str) -> Result<()> {
        if self.document["resources"].get(r).is_some() {
            Ok(())
        } else {
            Err(format!("§8.2: undeclared resource {r}"))
        }
    }
    fn op(&self, v: &Value) -> Result<()> {
        if [
            "x", "cx", "ccx", "mcx", "swap", "cz", "h", "s", "sdg", "t", "tdg", "rz", "rx", "ry",
        ]
        .contains(&text(v)?)
        {
            Ok(())
        } else {
            Err(format!("§8.3: unknown gate {v}"))
        }
    }
    fn expression(&self, v: &Value, vars: &[&str], nonnegative: bool) -> Result<()> {
        expr::check(v, vars, &self.document["tables"])?;
        if !contains_variable(v) {
            if nonnegative {
                expr::cost(v, &Bindings::new(), &self.document["tables"])?;
            } else {
                expr::eval(v, &Bindings::new(), &self.document["tables"])?;
            }
        }
        if let Some(args) = v.get("args").and_then(Value::as_array) {
            for a in args {
                self.expression(a, vars, false)?;
            }
        }
        Ok(())
    }
    fn costs(&self, v: &Value, vars: &[&str]) -> Result<()> {
        let c = obj(v)?;
        if !c.contains_key("time") {
            return Err("§8.2: gate occupies wires for its time; cost.time required".into());
        }
        for (r, e) in c {
            self.resource(r)?;
            self.expression(e, vars, true)?;
        }
        Ok(())
    }
    fn wire(&self, w: &str, vars: &[&str]) -> Result<()> {
        if ["t", "t2"].contains(&w) {
            return Ok(());
        }
        if (w.starts_with("a[") || w.starts_with("c[")) && w.ends_with(']') {
            let inner = &w[2..w.len() - 1];
            let e = json::parse(inner)
                .or_else(|_| Ok::<Value, String>(serde_json::json!({"var":inner})))?;
            return self.expression(&e, vars, false);
        }
        Err("§8.3: template wire must be c[expr], t, t2 or a[expr]".into())
    }
    pub fn native(
        &self,
        op: &str,
        classes: &[&str],
        k: usize,
        c: usize,
        eps: Rational,
    ) -> Result<Option<Charge>> {
        if c < k {
            return Err("§8.3: total controls must include own controls".into());
        }
        let env = Bindings::from([
            ("k".into(), Rational::from_integer(k.into())),
            ("c".into(), Rational::from_integer(c.into())),
            ("eps".into(), eps.clone()),
        ]);
        for g in arr(&self.document["gates"])? {
            if g["op"] != op || !covers(&g["controls"], c)? {
                continue;
            }
            let pattern = arr(&g["classes"])?;
            if pattern.len() != classes.len() && pattern.len() != 1 {
                continue;
            }
            if !classes.iter().enumerate().all(|(i, cls)| {
                let p = &pattern[if pattern.len() == 1 { 0 } else { i }];
                p == "*" || p == cls
            }) {
                continue;
            }
            let epsilon = expr::cost(&g["epsilon"], &env, &self.document["tables"])?;
            if epsilon > eps {
                continue;
            }
            if epsilon.is_zero() && (expr::uses_eps(&g["epsilon"]) || expr::uses_eps(&g["cost"])) {
                return Err(
                    "§8.1: an entry with epsilon 0 MUST NOT reference eps or log2inv_eps".into(),
                );
            }
            let cost = obj(&g["cost"])?
                .iter()
                .map(|(r, e)| Ok((r.clone(), expr::cost(e, &env, &self.document["tables"])?)))
                .collect::<Result<_>>()?;
            return Ok(Some(Charge { epsilon, cost }));
        }
        Ok(None)
    }
}
pub(crate) fn covers(v: &Value, c: usize) -> Result<bool> {
    Ok(c >= expr::integer(&v["min"])? && bound(&v["max"])?.is_none_or(|max| c <= max))
}
fn contains_variable(v: &Value) -> bool {
    match v {
        Value::Object(o) => {
            o.contains_key("var")
                || o.contains_key("table")
                || o.get("op").is_some_and(|x| x == "log2inv_eps")
                || o.values().any(contains_variable)
        }
        Value::Array(a) => a.iter().any(contains_variable),
        _ => false,
    }
}
