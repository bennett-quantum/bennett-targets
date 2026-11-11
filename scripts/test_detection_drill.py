"""Deliberate red: a detection drill (2026-11-11). Reverted with bennett/tools/revert_landing.py once seen."""
import unittest


class DetectionDrill(unittest.TestCase):
    def test_deliberate_red(self):
        self.fail("deliberate red: push-then-detect drill; revert this merge")
