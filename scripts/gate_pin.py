"""Read only gate-owned, pinned sibling checkouts; never live workspace siblings."""
from pathlib import Path
import tomllib


def pinned_sibling(root: Path, name: str) -> Path:
    try:
        with (root / "gate.toml").open("rb") as source:
            sha = tomllib.load(source)["siblings"][name]["sha"]
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"gate.toml: cannot read pin for {name}: {error}") from error
    if not isinstance(sha, str) or len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
        raise ValueError(f"gate.toml: invalid full SHA for {name}: {sha}")
    path = root / ".tmp/gate/cache" / f"{name}@{sha[:12]}" / name
    if not path.is_dir():
        raise FileNotFoundError(f"gate.toml: pinned sibling {name}@{sha[:12]} unavailable at {path}; populate the gate cache")
    return path
