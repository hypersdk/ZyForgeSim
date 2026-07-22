"""ForgeSim — thin Python API over the Rust simulation core."""

from typing import TYPE_CHECKING, Any

__all__ = ["SimResult", "run_from_config"]
__version__ = "0.1.0"

if TYPE_CHECKING:
    from forgesim._forgesim import SimResult


def __getattr__(name: str) -> Any:
    if name in ("SimResult", "run_from_config"):
        from forgesim import _forgesim

        return getattr(_forgesim, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
