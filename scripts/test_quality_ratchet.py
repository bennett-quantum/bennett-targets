import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("quality_ratchet.py")


class QualityRatchetTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "docs").mkdir()
        (self.root / "sample.rs").write_text(
            'fn sample(value: bool) {\n    if value {\n        action();\n    }\n}\n'
        )
        (self.root / "sample.py").write_text(
            'def sample(value):\n    if value:\n        action()\n'
        )
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "-C", str(self.root), "add", "sample.rs", "sample.py"], check=True)

    def invoke(self, *args):
        return subprocess.run(
            ["python3", str(SCRIPT), "--root", str(self.root), *args],
            capture_output=True, text=True, timeout=10,
        )

    def baseline(self):
        measured = self.invoke("--measure")
        self.assertEqual(measured.returncode, 0, measured.stderr)
        (self.root / "docs/quality.md").write_text(
            "<!-- quality-baseline -->\n```json\n" + measured.stdout + "```\n"
        )
        return json.loads(measured.stdout)

    def test_baseline_and_deliberately_longer_scratch_function(self):
        report = self.baseline()
        self.assertEqual(report["files"]["sample.rs"]["lengths"], [5])
        self.assertEqual(report["files"]["sample.rs"]["complexity"], 1)
        self.assertEqual(report["files"]["sample.py"]["lengths"], [3])
        self.assertEqual(self.invoke().returncode, 0)
        path = self.root / "sample.rs"
        path.write_text(path.read_text().replace("action();", "action();\n        more();"))
        failed = self.invoke()
        self.assertEqual(failed.returncode, 1, failed.stderr)
        self.assertIn("sample.rs: lengths", failed.stderr)

    def test_branch_mass_cannot_hide_in_same_length_function(self):
        self.baseline()
        (self.root / "sample.rs").write_text(
            'fn sample(value: bool) {\n    if value {\n        if value { action(); }\n    }\n}\n'
        )
        failed = self.invoke()
        self.assertEqual(failed.returncode, 1, failed.stderr)
        self.assertIn("sample.rs: complexity 1 -> 3", failed.stderr)

    def test_shorter_forwarder_still_fails_wrapper_ratchet(self):
        self.baseline()
        (self.root / "sample.rs").write_text("fn sample(value: bool) { callee(value); }\n")
        failed = self.invoke()
        self.assertEqual(failed.returncode, 1, failed.stderr)
        self.assertIn("sample.rs: wrappers 0 -> 1", failed.stderr)
        self.assertNotIn("sample.rs: lengths", failed.stderr)

    def test_structural_improvement_passes(self):
        self.baseline()
        (self.root / "sample.rs").write_text("fn sample() {}\n")
        passed = self.invoke()
        self.assertEqual(passed.returncode, 0, passed.stderr)

    def test_new_file_wrapper_litter_and_process_identifiers_fail(self):
        self.baseline()
        (self.root / "new.py").write_text("def forward(value):\n    return callee(value)\n")
        (self.root / "bad.log").write_text("generated\n")
        with (self.root / "sample.rs").open("a") as source:
            source.write("// b-targets-example job/quality-example shift 12\n")
        subprocess.run(["git", "-C", str(self.root), "add", "new.py", "bad.log"], check=True)
        failed = self.invoke()
        self.assertEqual(failed.returncode, 1, failed.stderr)
        for diagnostic in ["new.py: lengths", "new.py: wrappers", "bad.log", "sample.rs: process identifiers"]:
            self.assertIn(diagnostic, failed.stderr)

    def test_rust_literals_comments_and_nested_functions(self):
        (self.root / "sample.rs").write_text(
            'fn outer() {\n    /* if { /* while { */ } */\n'
            '    let text = r###"if { fn fake() {}"###;\n'
            '    fn inner() {\n        if true { action(); }\n    }\n}\n'
        )
        report = self.baseline()["files"]["sample.rs"]
        self.assertEqual(report, {"lengths": [7, 3], "complexity": 1, "wrappers": 0})

    def test_python_nested_branches_and_wrapper(self):
        (self.root / "sample.py").write_text(
            'def outer(value):\n    if value:\n        while value and other:\n            action()\n'
            'def forward(value):\n    return callee(value)\n'
        )
        report = self.baseline()["files"]["sample.py"]
        self.assertEqual(report, {"lengths": [4, 2], "complexity": 4, "wrappers": 1})

    def test_top_level_python_control_flow_and_rust_forwarder(self):
        (self.root / "sample.py").write_text("if ready:\n    for item in items:\n        action(item)\n")
        (self.root / "sample.rs").write_text("fn forward(value: bool) { callee(value); }\n")
        report = self.baseline()["files"]
        self.assertEqual(report["sample.py"], {"lengths": [], "complexity": 3, "wrappers": 0})
        self.assertEqual(report["sample.rs"], {"lengths": [1], "complexity": 0, "wrappers": 1})

    def test_missing_baseline_and_invalid_source_fail_loudly(self):
        failed = self.invoke()
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn("quality.md", failed.stderr)
        (self.root / "sample.py").write_text("def broken(:\n")
        failed = self.invoke("--measure")
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn("sample.py", failed.stderr)
