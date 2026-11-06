use super::pins;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SCRATCH: AtomicUsize = AtomicUsize::new(0);

struct Sibling {
    scratch: PathBuf,
    root: PathBuf,
    layout: PathBuf,
    path: PathBuf,
    sha: String,
    config: String,
}

impl Sibling {
    fn new() -> Self {
        let scratch = std::env::temp_dir().join(format!(
            "pin-test-{}-{}",
            std::process::id(),
            NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed)
        ));
        let root = scratch.join("targets");
        let layout = scratch.join("run");
        let path = layout.join("isa");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&path).unwrap();
        let mut fixture = Self {
            scratch,
            root,
            layout,
            path,
            sha: String::new(),
            config: String::new(),
        };
        fixture.git(&["init", "--quiet"]);
        fixture.git(&["commit", "--quiet", "--allow-empty", "-m", "pinned"]);
        fixture.sha = fixture.git(&["rev-parse", "HEAD"]);
        fixture.config = format!("[siblings.\"isa\"]\nsha = \"{}\"\n", fixture.sha);
        fixture
    }

    fn git(&self, args: &[&str]) -> String {
        let mut command = Command::new("git");
        for (key, _) in
            std::env::vars_os().filter(|(key, _)| key.to_string_lossy().starts_with("GIT_"))
        {
            command.env_remove(key);
        }
        let output = command
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "user.name=Pin Test",
                "-c",
                "user.email=pin@example.invalid",
                "-C",
            ])
            .arg(&self.path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn move_to_cache(&mut self) {
        let cache = self
            .root
            .join(".tmp/gate/cache")
            .join(format!("isa@{}", &self.sha[..12]))
            .join("isa");
        std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
        std::fs::rename(&self.path, &cache).unwrap();
        self.path = cache;
    }
}

impl Drop for Sibling {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.scratch).unwrap();
    }
}

#[test]
fn pinned_sibling_resolves_verified_cache_and_gate_layout() {
    for use_layout in [false, true] {
        let mut fixture = Sibling::new();
        if !use_layout {
            fixture.move_to_cache();
        }
        let layout = use_layout.then_some(fixture.layout.as_path());
        assert_eq!(
            pins::resolve(&fixture.root, &fixture.config, "isa", layout).unwrap(),
            fixture.path
        );
    }
}

#[test]
fn pinned_sibling_rejects_mismatched_head_in_both_layouts() {
    for use_layout in [false, true] {
        let mut fixture = Sibling::new();
        if !use_layout {
            fixture.move_to_cache();
        }
        fixture.git(&["commit", "--quiet", "--allow-empty", "-m", "moved"]);
        let actual = fixture.git(&["rev-parse", "HEAD"]);
        let layout = use_layout.then_some(fixture.layout.as_path());
        let error = pins::resolve(&fixture.root, &fixture.config, "isa", layout).unwrap_err();
        for detail in [
            fixture.sha.clone(),
            actual,
            fixture.path.display().to_string(),
            fixture
                .root
                .parent()
                .unwrap()
                .join("isa")
                .display()
                .to_string(),
        ] {
            assert!(error.contains(&detail), "{error} lacks {detail}");
        }
    }
}

#[test]
fn pinned_sibling_missing_layout_never_uses_matching_cache_or_live_sibling() {
    let mut fixture = Sibling::new();
    let live = fixture.root.parent().unwrap().join("isa");
    fixture.git(&[
        "clone",
        "--quiet",
        fixture.path.to_str().unwrap(),
        live.to_str().unwrap(),
    ]);
    fixture.move_to_cache();
    let error =
        pins::resolve(&fixture.root, &fixture.config, "isa", Some(&fixture.layout)).unwrap_err();
    assert!(
        error.contains(&fixture.layout.join("isa").display().to_string()),
        "{error}"
    );
}

#[test]
fn pinned_sibling_directory_without_git_metadata_is_rejected() {
    let mut fixture = Sibling::new();
    fixture.move_to_cache();
    std::fs::remove_dir_all(fixture.path.join(".git")).unwrap();
    let error = pins::resolve(&fixture.root, &fixture.config, "isa", None).unwrap_err();
    assert!(error.contains("HEAD"), "{error}");
}

#[test]
fn pinned_sibling_never_falls_back_to_a_live_checkout() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let config = std::fs::read_to_string(root.join("gate.toml")).unwrap();
    let missing = config
        .lines()
        .map(|line| {
            if line.starts_with("sha = ") {
                format!("sha = \"{}\"", "0".repeat(40))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let error = pins::resolve(&root, &missing, "bennett-isa-spec", None).unwrap_err();
    assert!(error.contains("bennett-isa-spec@000000000000"), "{error}");
    assert!(error.contains("gate.toml"), "{error}");
    assert!(pins::isa().join("spec/Bennett-ISA-Spec-0.9.md").is_file());
}
