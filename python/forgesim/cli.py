"""Optional CLI entry point when running without maturin develop."""

import sys


def main() -> None:
    print(
        "Use the Rust CLI: cargo run -p forgesim-cli -- run --config <path>",
        file=sys.stderr,
    )
    print(
        "Or from Python: import forgesim; forgesim.run_from_config('<path>')",
        file=sys.stderr,
    )
    sys.exit(1)


if __name__ == "__main__":
    main()
