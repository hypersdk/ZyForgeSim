"""Gymnasium environments wrapping ForgeSim RL sessions."""

from typing import TYPE_CHECKING, Any

__all__ = ["ForgeSimEnv"]

if TYPE_CHECKING:
    from forgesim.envs.forge_gym import ForgeSimEnv


def __getattr__(name: str) -> Any:
    if name == "ForgeSimEnv":
        from forgesim.envs.forge_gym import ForgeSimEnv

        return ForgeSimEnv
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
