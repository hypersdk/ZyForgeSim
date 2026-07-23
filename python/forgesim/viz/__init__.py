"""Visualization helpers for ForgeSim run outputs."""

from typing import TYPE_CHECKING, Any

__all__ = ["load_timeline", "plot_gantt", "plot_gpu_heatmap", "save_run_figures"]

if TYPE_CHECKING:
    from pathlib import Path

    from matplotlib.figure import Figure


def __getattr__(name: str) -> Any:
    if name == "load_timeline":
        from forgesim.viz.timeline import load_timeline

        return load_timeline
    if name == "plot_gantt":
        from forgesim.viz.gantt import plot_gantt

        return plot_gantt
    if name == "plot_gpu_heatmap":
        from forgesim.viz.heatmap import plot_gpu_heatmap

        return plot_gpu_heatmap
    if name == "save_run_figures":
        from forgesim.viz.plots import save_run_figures

        return save_run_figures
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
