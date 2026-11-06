"""Read verified sibling pins from the gate layout or its materialised cache."""
import os
from pathlib import Path
import subprocess
import tomllib


def pinned_sibling(root: Path, name: str) -> Path:
    try:
        with (root / "gate.toml").open("rb") as source:
            sha = tomllib.load(source)["siblings"][name]["sha"]
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"gate.toml: cannot read pin for {name}: {error}") from error
    if not isinstance(sha, str) or len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError(f"gate.toml: invalid full SHA for {name}: {sha}")
    layout = os.environ.get("BENNETT_GATE_LAYOUT")
    path = (Path(layout) / name if layout is not None else
            root / ".tmp/gate/cache" / f"{name}@{sha[:12]}" / name)
    context = (f"gate.toml: pinned sibling {name}@{sha[:12]} unavailable at {path}; "
               f"expected HEAD={sha}; live sibling {root.parent / name} is never used")
    if not path.is_dir():
        raise FileNotFoundError(context)
    if not (path / ".git").exists():
        raise ValueError(f"{context}; HEAD unavailable: missing .git metadata")
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    try:
        head = subprocess.run(["git", "-C", str(path), "rev-parse", "--verify", "HEAD^{commit}"],
                              capture_output=True, text=True, env=environment, timeout=10, check=True)
    except (OSError, subprocess.SubprocessError) as error:
        raise ValueError(f"{context}; cannot read HEAD: {error}") from error
    actual = head.stdout.strip()
    if actual != sha:
        raise ValueError(f"{context}; actual HEAD={actual}")
    return path
