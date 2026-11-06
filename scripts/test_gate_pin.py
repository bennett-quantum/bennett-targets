import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from gate_pin import pinned_sibling


class PinnedSiblingTests(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name) / "targets"
        self.root.mkdir()
        self.layout = Path(scratch.name) / "run"
        self.sibling = self.layout / "isa"
        self.sibling.mkdir(parents=True)
        self.git("init", "--quiet")
        self.git("commit", "--quiet", "--allow-empty", "-m", "pinned")
        self.sha = self.git("rev-parse", "HEAD")
        (self.root / "gate.toml").write_text(f'[siblings."isa"]\nsha = "{self.sha}"\n')
        self.cache = self.root / ".tmp/gate/cache" / f"isa@{self.sha[:12]}" / "isa"
        environment = patch.dict(os.environ)
        environment.start()
        self.addCleanup(environment.stop)
        os.environ.pop("BENNETT_GATE_LAYOUT", None)

    def git(self, *args):
        output = subprocess.check_output(
            ["git", "-c", "core.hooksPath=/dev/null", "-c", "user.name=Pin Test",
             "-c", "user.email=pin@example.invalid", "-C", str(self.sibling), *args],
            text=True, env={k: v for k, v in os.environ.items() if not k.startswith("GIT_")},
            timeout=10,
        )
        return output.strip()

    def move_to_cache(self):
        self.cache.parent.mkdir(parents=True)
        self.sibling.rename(self.cache)
        self.sibling = self.cache

    def test_resolves_only_the_configured_cache(self):
        self.move_to_cache()
        self.assertEqual(pinned_sibling(self.root, "isa"), self.cache)

    def test_resolves_gate_layout_without_a_cache(self):
        os.environ["BENNETT_GATE_LAYOUT"] = str(self.layout)
        self.assertFalse(self.cache.exists())
        self.assertEqual(pinned_sibling(self.root, "isa"), self.sibling)

    def test_mismatched_head_names_expected_actual_and_both_paths(self):
        for layout in (None, self.layout):
            with self.subTest(layout=layout):
                if layout is None:
                    self.move_to_cache()
                else:
                    self.sibling.rename(self.layout / "isa")
                    self.sibling = self.layout / "isa"
                    os.environ["BENNETT_GATE_LAYOUT"] = str(layout)
                self.git("commit", "--quiet", "--allow-empty", "-m", "moved")
                actual = self.git("rev-parse", "HEAD")
                with self.assertRaises(ValueError) as caught:
                    pinned_sibling(self.root, "isa")
                for detail in (self.sha, actual, str(self.sibling), str(self.root.parent / "isa")):
                    self.assertIn(detail, str(caught.exception))

    def test_missing_cache_rejects_even_with_a_matching_live_sibling(self):
        self.sibling.rename(self.root.parent / "isa")
        with self.assertRaisesRegex(FileNotFoundError, "gate.toml: pinned sibling isa@"):
            pinned_sibling(self.root, "isa")

    def test_missing_layout_never_uses_matching_cache_or_live_sibling(self):
        shutil.copytree(self.sibling, self.root.parent / "isa")
        self.move_to_cache()
        os.environ["BENNETT_GATE_LAYOUT"] = str(self.layout)
        with self.assertRaisesRegex(FileNotFoundError, str(self.layout / "isa")):
            pinned_sibling(self.root, "isa")

    def test_directory_without_git_metadata_is_not_a_pin(self):
        self.move_to_cache()
        shutil.rmtree(self.cache / ".git")
        with self.assertRaisesRegex(ValueError, "HEAD"):
            pinned_sibling(self.root, "isa")

    def test_malformed_and_absent_pins_name_the_config_and_sibling(self):
        for config in ['[siblings."isa"]\nsha = 123', '[siblings."isa"]\nsha = "abc"',
                       '[siblings."isa"]\nsha = "' + "g" * 40 + '"', "schema = 1", "[invalid"]:
            with self.subTest(config=config):
                (self.root / "gate.toml").write_text(config)
                with self.assertRaisesRegex(ValueError, "gate.toml: .*isa"):
                    pinned_sibling(self.root, "isa")


if __name__ == "__main__":
    unittest.main()
