#!/usr/bin/env python3
"""Run a ForgeSim FIFO simulation from the sample H100 cluster config."""

from pathlib import Path

import forgesim

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / "configs" / "clusters" / "small_h100.yaml"


def main() -> None:
    result = forgesim.run_from_config(str(CONFIG))
    print(result)
    print(result.to_json())


if __name__ == "__main__":
    main()
