use std::path::{Path, PathBuf};

/// The gate owns this cache; tests must never consult a live sibling checkout.
pub fn resolve(root: &Path, config: &str, name: &str) -> Result<PathBuf, String> {
    let section = format!("[siblings.\"{name}\"]");
    let sha = config
        .split_once(&section)
        .and_then(|(_, rest)| {
            rest.lines()
                .take_while(|line| !line.trim().starts_with('['))
                .find_map(|line| line.trim().strip_prefix("sha = \""))
        })
        .and_then(|value| value.strip_suffix('"'))
        .filter(|sha| sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| format!("gate.toml: missing full SHA for {name}"))?;
    let path = root
        .join(".tmp/gate/cache")
        .join(format!("{name}@{}", &sha[..12]))
        .join(name);
    if !path.is_dir() {
        return Err(format!(
            "gate.toml: pinned sibling {name}@{} unavailable at {}; populate the gate cache",
            &sha[..12],
            path.display()
        ));
    }
    Ok(path)
}

pub fn isa() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let config = std::fs::read_to_string(root.join("gate.toml")).expect("read gate.toml");
    resolve(&root, &config, "bennett-isa-spec").unwrap_or_else(|error| panic!("{error}"))
}
