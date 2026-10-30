//! Strict JSON input with duplicate rejection, and RFC 8785 output.
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
pub type Result<T> = std::result::Result<T, String>;
pub fn parse(s: &str) -> Result<Value> {
    struct Parser<'a> { s: &'a str, i: usize }
    impl Parser<'_> {
        fn ws(&mut self) { while self.s.as_bytes().get(self.i).is_some_and(|c| b" \r\n\t".contains(c)) { self.i += 1; } }
        fn string(&mut self) -> Result<String> {
            let start = self.i; self.i += 1;
            while let Some(&b) = self.s.as_bytes().get(self.i) {
                self.i += 1;
                if b == b'\\' { self.i += 1; }
                if b == b'"' { return serde_json::from_str(&self.s[start..self.i]).map_err(|e|e.to_string()); }
            }
            Err("unterminated string".into())
        }
        fn value(&mut self, depth: usize) -> Result<Value> {
            if depth > 256 { return Err("host limit: JSON nesting >256".into()); }
            self.ws();
            match self.s.as_bytes().get(self.i).copied() {
                Some(b'"') => Ok(Value::String(self.string()?)),
                Some(b'{') | Some(b'[') => {
                    let object = self.s.as_bytes()[self.i] == b'{'; self.i += 1;
                    let end = if object { b'}' } else { b']' };
                    let mut map = Map::new(); let mut arr = Vec::new(); self.ws();
                    if self.s.as_bytes().get(self.i) != Some(&end) {
                        loop {
                            self.ws();
                            if object {
                                if self.s.as_bytes().get(self.i) != Some(&b'"') { return Err("expected object key".into()); }
                                let key = self.string()?; self.ws();
                                if self.s.as_bytes().get(self.i) != Some(&b':') { return Err("expected colon".into()); } self.i += 1;
                                let v = self.value(depth+1)?;
                                if map.insert(key.clone(), v).is_some() { return Err(format!("duplicate key {key}")); }
                            } else { arr.push(self.value(depth+1)?); }
                            self.ws();
                            if self.s.as_bytes().get(self.i) == Some(&end) { break; }
                            if self.s.as_bytes().get(self.i) != Some(&b',') { return Err("expected comma".into()); } self.i += 1;
                        }
                    }
                    self.i += 1;
                    Ok(if object { Value::Object(map) } else { Value::Array(arr) })
                }
                Some(_) => {
                    let start = self.i;
                    while self.s.as_bytes().get(self.i).is_some_and(|c| !b",]} \r\n\t".contains(c)) { self.i += 1; }
                    serde_json::from_str(&self.s[start..self.i]).map_err(|e|e.to_string())
                }
                None => Err("unexpected end of JSON".into())
            }
        }
    }
    let mut p = Parser {s, i:0}; let v = p.value(0).map_err(|e| format!("§8.1: JSON document: {e}"))?; p.ws();
    if p.i != s.len() { return Err("§8.1: trailing JSON input".into()); } Ok(v)
}
pub fn canonical(v: &Value) -> Result<String> {
    Ok(match v {
        Value::Null => "null".into(), Value::Bool(b) => b.to_string(),
        Value::String(s) => serde_json::to_string(s).map_err(|e|e.to_string())?,
        Value::Number(n) => {
            let f: f64 = n.to_string().parse().map_err(|_|"§8.1: RFC 8785 number")?;
            if !f.is_finite() { return Err("§8.1: RFC 8785 numbers must be finite binary64".into()); }
            ryu_js::Buffer::new().format(f).to_string()
        }
        Value::Array(a) => format!("[{}]", a.iter().map(canonical).collect::<Result<Vec<_>>>()?.join(",")),
        Value::Object(o) => {
            let mut keys: Vec<_> = o.keys().collect(); keys.sort_by_cached_key(|k|k.encode_utf16().collect::<Vec<_>>());
            format!("{{{}}}", keys.into_iter().map(|k|Ok(format!("{}:{}", canonical(&Value::String(k.clone()))?, canonical(&o[k])?))).collect::<Result<Vec<_>>>()?.join(","))
        }
    })
}
pub fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
