import unittest

from consumer_report import stable_report


class ConsumerReportTests(unittest.TestCase):
    def test_nested_upstream_reasons_keep_contract_text_and_status(self):
        original = {"state": "BLOCKED", "rows": [{"reason": "§8.3 topology unavailable (b-targets-example)"}, "b-isa-spec-example-extra"], "count": 0}
        result = stable_report(original)
        self.assertEqual(result, {"state": "BLOCKED", "rows": [{"reason": "§8.3 topology unavailable (tracked contract issue)"}, "tracked contract issue"], "count": 0})
        self.assertEqual(original["rows"][0]["reason"], "§8.3 topology unavailable (b-targets-example)")

    def test_scenario_identifiers_and_ordinary_diagnostics_are_unchanged(self):
        value = {"id": "E-topology-grid-09", "reason": "E_SCHEDULE_INFEASIBLE: §17 item 15", "valid": False, "details": None}
        self.assertEqual(stable_report(value), value)


if __name__ == "__main__":
    unittest.main()
