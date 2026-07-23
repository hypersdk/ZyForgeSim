"""Plot Gantt and GPU heatmap from a ForgeSim jobs timeline JSON."""

from __future__ import annotations

import argparse
from pathlib import Path

from forgesim.viz import save_run_figures


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("timeline", type=Path, help="jobs timeline JSON from forge-sim run")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("outputs/figures"),
        help="directory for PNG outputs",
    )
    parser.add_argument("--prefix", default="run")
    args = parser.parse_args()

    gantt, heatmap = save_run_figures(args.timeline, args.output_dir, prefix=args.prefix)
    print(f"wrote {gantt}")
    print(f"wrote {heatmap}")


if __name__ == "__main__":
    main()
