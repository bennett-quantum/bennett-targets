from pathlib import Path
import tempfile
import unittest

from gate_pin import pinned_sibling


class PinnedSiblingTests(unittest.TestCase):
    def test_resolves_only_the_configured_cache(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            sha = "1234567890abcdef" + "a" * 24
            (root / "gate.toml").write_text(f'[siblings."isa"]\nsha = "{sha}"\n')
            cache = root / ".tmp/gate/cache" / f"isa@{sha[:12]}" / "isa"
            cache.mkdir(parents=True)
            self.assertEqual(pinned_sibling(root, "isa"), cache)

    def test_missing_cache_rejects_even_with_a_live_sibling(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch) / "targets"
            root.mkdir()
            (root.parent / "isa").mkdir()
            (root / "gate.toml").write_text('[siblings."isa"]\nsha = "' + "a" * 40 + '"\n')
            with self.assertRaisesRegex(FileNotFoundError, "gate.toml: pinned sibling isa@aaaaaaaaaaaa unavailable"):
                pinned_sibling(root, "isa")

    def test_malformed_and_absent_pins_name_the_config_and_sibling(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            for config in ['[siblings."isa"]\nsha = 123', '[siblings."isa"]\nsha = "abc"', '[siblings."isa"]\nsha = "' + "g" * 40 + '"', "schema = 1", "[invalid"]:
                with self.subTest(config=config):
                    (root / "gate.toml").write_text(config)
                    with self.assertRaisesRegex(ValueError, "gate.toml: .*isa"):
                        pinned_sibling(root, "isa")


if __name__ == "__main__":
    unittest.main()
