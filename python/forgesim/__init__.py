"""ForgeSim — thin Python API over the Rust simulation core."""

from typing import TYPE_CHECKING, Any

__all__ = ["SimResult", "SimSession", "run_from_config", "ForgeSimEnv"]
__version__ = "0.1.0"

if TYPE_CHECKING:
    from forgesim._forgesim import SimResult, SimSession
    from forgesim.envs.forge_gym import ForgeSimEnv


def __getattr__(name: str) -> Any:
    if name in ("SimResult", "SimSession", "run_from_config"):
        from forgesim import _forgesim

        return getattr(_forgesim, name)
    if name == "ForgeSimEnv":
        from forgesim.envs.forge_gym import ForgeSimEnv

        return ForgeSimEnv
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
