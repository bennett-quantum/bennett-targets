//! ISA 0.9 §8.3 topology, §11 default placement, §17 item 15 infeasibility.
use crate::{expr, json::Result, runtime::Leaf, Target};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn array<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| format!("§8.3 topology: {what} must be an array"))
}

impl Target {
    pub(crate) fn topology_schema(&self, t: &Value) -> Result<()> {
        let o = t.as_object().ok_or("§8.3 topology: expected object")?;
        for k in ["vertices", "edges"] {
            if !o.contains_key(k) {
                return Err(format!("§8.3 topology schema: missing {k}"));
            }
        }
        if let Some(k) = o.keys().find(|k| *k != "vertices" && *k != "edges") {
            return Err(format!("§8.3 topology schema: unknown field {k}"));
        }
        let vertices = array(&t["vertices"], "vertices")?;
        for v in vertices {
            let v = v
                .as_object()
                .ok_or("§8.3 topology: vertex must be object")?;
            if v.len() != 1 {
                return Err("§8.3 topology schema: vertex has exactly the field class".into());
            }
            let class = v
                .get("class")
                .and_then(Value::as_str)
                .ok_or("§8.3 topology: vertex class string required")?;
            self.class(class)?;
        }
        let mut previous: Option<(usize, usize)> = None;
        for e in array(&t["edges"], "edges")? {
            let e = array(e, "edge")?;
            if e.len() != 2 {
                return Err("§8.3 topology: an edge is a pair [u,v]".into());
            }
            let (u, v) = (expr::integer(&e[0])?, expr::integer(&e[1])?);
            if u >= v || v >= vertices.len() {
                return Err(format!(
                    "§8.3 topology: edge [{u},{v}] needs 0 <= u < v < vertices.length"
                ));
            }
            if previous.is_some_and(|p| p >= (u, v)) {
                return Err(format!(
                    "§8.3 topology: edges MUST be sorted lexicographically, without duplicates: [{u},{v}]"
                ));
            }
            previous = Some((u, v));
        }
        Ok(())
    }

    /// Whether one native gate may act on these operand vertices, in operand
    /// order including all effective control wires. Absent topology is all-to-all.
    pub fn check_placement(&self, vertices: &[usize]) -> Result<()> {
        if vertices.iter().collect::<BTreeSet<_>>().len() != vertices.len() {
            return Err(format!(
                "§8.3: gate operands occupy distinct wires and vertices: {vertices:?}"
            ));
        }
        let Some(t) = self.document.get("topology") else {
            return Ok(());
        };
        let count = array(&t["vertices"], "vertices")?.len();
        if let Some(v) = vertices.iter().find(|v| **v >= count) {
            return Err(format!("§8.3 topology: no vertex {v} in {count} vertices"));
        }
        let edges = array(&t["edges"], "edges")?
            .iter()
            .map(|e| Ok((expr::integer(&e[0])?, expr::integer(&e[1])?)))
            .collect::<Result<BTreeSet<_>>>()?;
        for (i, a) in vertices.iter().enumerate() {
            for b in &vertices[i + 1..] {
                if !edges.contains(&(*a.min(b), *a.max(b))) {
                    return Err(format!(
                        "E_SCHEDULE_INFEASIBLE: §17 item 15: operand vertices {a} and {b} are not adjacent; no routing is inserted (§11)"
                    ));
                }
            }
        }
        Ok(())
    }

    /// Check every native leaf of an expansion; `placement` maps the leaf wire
    /// numbering of `expand` (controls, targets, then ancillae) to vertices.
    /// Each wire must sit on a vertex of its resolved class (§8.3 "Each live wire
    /// occupies one vertex of its resolved class"); the expansion's wires are
    /// live together, so they occupy distinct vertices.
    pub fn check_leaves(&self, leaves: &[Leaf], placement: &[usize]) -> Result<()> {
        let mut seen = BTreeMap::new();
        for leaf in leaves {
            for w in &leaf.wires {
                if let Some(v) = placement.get(*w) {
                    if let Some(other) = seen.insert(*v, *w).filter(|o| o != w) {
                        return Err(format!(
                            "§8.3: wires {other} and {w} must occupy distinct vertices, both placed on {v}"
                        ));
                    }
                }
            }
        }
        let vertex_class = |v: usize| -> Result<Option<String>> {
            let Some(t) = self.document.get("topology") else {
                return Ok(None);
            };
            Ok(array(&t["vertices"], "vertices")?
                .get(v)
                .and_then(|x| x["class"].as_str())
                .map(str::to_string))
        };
        for leaf in leaves {
            if leaf.classes.len() != leaf.wires.len() {
                return Err(format!(
                    "§8.3: leaf {} has {} wires but {} resolved classes",
                    leaf.op,
                    leaf.wires.len(),
                    leaf.classes.len()
                ));
            }
            let vertices = leaf
                .wires
                .iter()
                .map(|w| {
                    placement.get(*w).copied().ok_or_else(|| {
                        format!("§11 placement: no vertex for leaf wire {w} of {}", leaf.op)
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            self.check_placement(&vertices)?;
            for ((w, v), class) in leaf.wires.iter().zip(&vertices).zip(&leaf.classes) {
                if let Some(vc) = vertex_class(*v)? {
                    if &vc != class {
                        return Err(format!(
                            "§8.3/§11 placement: wire {w} of {} has resolved class {class} but vertex {v} has class {vc}",
                            leaf.op
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// §11 omitted placements: each request (class, width), in order, takes the
    /// lowest-index free vertices of its class, assigning bits in order.
    pub fn default_placement(&self, requests: &[(&str, usize)]) -> Result<Vec<Vec<usize>>> {
        let t = self
            .document
            .get("topology")
            .ok_or("§11: placements MUST be absent without a topology")?;
        let vertices = array(&t["vertices"], "vertices")?;
        let mut used = BTreeSet::new();
        let mut out = Vec::new();
        for (class, width) in requests {
            self.class(class)?;
            let mut bits = Vec::new();
            for _ in 0..*width {
                let free = (0..vertices.len())
                    .find(|i| !used.contains(i) && vertices[*i]["class"] == **class)
                    .ok_or_else(|| {
                        format!("§8.3: vertices of class {class} limit occupancy; none free")
                    })?;
                used.insert(free);
                bits.push(free);
            }
            out.push(bits);
        }
        Ok(out)
    }

    /// Vertices per class when a topology is declared.
    pub(crate) fn vertex_limits(&self) -> Result<Option<BTreeMap<String, usize>>> {
        let Some(t) = self.document.get("topology") else {
            return Ok(None);
        };
        let mut limits = BTreeMap::new();
        for v in array(&t["vertices"], "vertices")? {
            let class = v["class"].as_str().ok_or("§8.3 topology: vertex class")?;
            *limits.entry(class.to_string()).or_insert(0) += 1;
        }
        Ok(Some(limits))
    }
}
