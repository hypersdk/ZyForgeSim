"""Unit tests for scheduling metrics exposed via the Python bindings."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PREEMPTIVE_CONFIG = ROOT / "configs/clusters/preemption_preemptive.yaml"


@unittest.skipUnless(PREEMPTIVE_CONFIG.exists(), "preemptive config missing")
class TestSchedulingMetrics(unittest.TestCase):
    def test_preemptive_run_reports_segment_metrics(self) -> None:
        try:
            import forgesim
            from forgesim import _forgesim
        except ImportError:
            self.skipTest("forgesim extension not built")

        metrics = _forgesim.run_from_config(str(PREEMPTIVE_CONFIG))
        self.assertEqual(metrics.preemptions, 1)
        self.assertGreater(metrics.gpu_utilization, 0.0)
        self.assertGreaterEqual(metrics.jobs_completed, 1)

    def test_sim_result_json_includes_new_fields(self) -> None:
        try:
            from forgesim import _forgesim
        except ImportError:
            self.skipTest("forgesim extension not built")

        metrics = _forgesim.run_from_config(str(PREEMPTIVE_CONFIG))
        payload = metrics.to_json()
        self.assertIn("jobs_unschedulable", payload)
        self.assertIn("queue_max_length", payload)


if __name__ == "__main__":
    unittest.main()
