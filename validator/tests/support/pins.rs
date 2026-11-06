use std::path::{Path, PathBuf};
use std::process::Command;

/// Only use the gate's explicit layout or cache, with HEAD verified against its pin.
pub fn resolve(
    root: &Path,
    config: &str,
    name: &str,
    layout: Option<&Path>,
) -> Result<PathBuf, String> {
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
    let root = root
        .canonicalize()
        .map_err(|error| format!("gate.toml: {}: {error}", root.display()))?;
    let live = root
        .parent()
        .ok_or_else(|| format!("gate.toml: {} has no parent", root.display()))?
        .join(name);
    let path = match layout {
        Some(layout) => layout.join(name),
        None => root
            .join(".tmp/gate/cache")
            .join(format!("{name}@{}", &sha[..12]))
            .join(name),
    };
    let context = format!(
        "gate.toml: pinned sibling {name}@{} unavailable at {}; expected HEAD={sha}; live sibling {} is never used",
        &sha[..12], path.display(), live.display()
    );
    if !path.is_dir() {
        return Err(context);
    }
    if !path.join(".git").exists() {
        return Err(format!(
            "{context}; HEAD unavailable: missing .git metadata"
        ));
    }
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os().filter(|(key, _)| key.to_string_lossy().starts_with("GIT_"))
    {
        command.env_remove(key);
    }
    let head = command
        .arg("-C")
        .arg(&path)
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .output()
        .map_err(|error| format!("{context}; cannot read HEAD: {error}"))?;
    if !head.status.success() {
        return Err(format!(
            "{context}; cannot read HEAD: {}",
            String::from_utf8_lossy(&head.stderr).trim()
        ));
    }
    let actual = String::from_utf8(head.stdout)
        .map_err(|error| format!("{context}; invalid HEAD: {error}"))?;
    if actual.trim() != sha {
        return Err(format!("{context}; actual HEAD={}", actual.trim()));
    }
    Ok(path)
}

pub fn isa() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let config = std::fs::read_to_string(root.join("gate.toml")).expect("read gate.toml");
    let layout = std::env::var_os("BENNETT_GATE_LAYOUT").map(PathBuf::from);
    resolve(&root, &config, "bennett-isa-spec", layout.as_deref())
        .unwrap_or_else(|error| panic!("{error}"))
}
