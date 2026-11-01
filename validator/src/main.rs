use bennett_targets_validator::{canon, importers, json, Target};
use serde_json::json;
mod query;
fn run() -> Result<serde_json::Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 || args.len() > 3 {
        return Err("usage: bennett-targets-validator validate|canon <absolute-file> | eval|adjacency <target> <query-json> | import-qualtran|import-azure <base-target> <source-json>".into());
    }
    if args.len() == 3
        && !args[0].starts_with("import-")
        && !["eval", "adjacency"].contains(&args[0].as_str())
    {
        return Err("unexpected third argument".into());
    }
    let text = std::fs::read_to_string(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
    match args[0].as_str() {
        "import-qualtran" | "import-azure" => {
            if args.len() != 3 {
                return Err("import requires <base-target> <source-json>".into());
            }
            let base = Target::load(&text)?;
            let input =
                json::parse(&std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?)?;
            if args[0] == "import-qualtran" {
                importers::qualtran(&base, &input)
            } else {
                importers::azure(&base, &input)
            }
        }
        "eval" | "adjacency" => {
            if args.len() != 3 {
                return Err(format!("{} requires <target> <query-json>", args[0]));
            }
            let t = Target::load(&text)?;
            let q = json::parse(&std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?)?;
            if args[0] == "eval" {
                query::eval(&t, &q)
            } else {
                query::adjacency(&t, &q)
            }
        }
        "validate" => {
            let t = Target::load(&text)?;
            Ok(json!({"valid":true,"target_hash":t.hash}))
        }
        "canon" => {
            let (bytes, hash) = canon(&text)?;
            Ok(json!({"canonical":bytes,"target_hash":hash}))
        }
        _ => Err("unknown command; expected validate or canon".into()),
    }
}
fn main() {
    match run() {
        Ok(v) => println!("{}", json::canonical(&v).expect("CLI JSON")),
        Err(e) => {
            eprintln!("{}", json!({"error":e}));
            std::process::exit(1);
        }
    }
}
