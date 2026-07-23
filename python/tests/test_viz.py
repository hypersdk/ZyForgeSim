"""Tests for ForgeSim visualization helpers."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests/fixtures/viz/jobs_small.json"


class TestViz(unittest.TestCase):
    def test_load_timeline(self) -> None:
        from forgesim.viz.timeline import load_timeline

        data = load_timeline(FIXTURE)
        self.assertEqual(len(data["jobs"]), 2)
        self.assertEqual(data["gpu_count"], 2)

    def test_save_run_figures(self) -> None:
        try:
            __import__("matplotlib")
        except ImportError:
            self.skipTest("matplotlib not installed")

        from forgesim.viz.plots import save_run_figures

        out_dir = ROOT / "outputs/test_viz"
        gantt, heatmap = save_run_figures(FIXTURE, out_dir, prefix="test")
        self.assertTrue(gantt.exists())
        self.assertTrue(heatmap.exists())


if __name__ == "__main__":
    unittest.main()
