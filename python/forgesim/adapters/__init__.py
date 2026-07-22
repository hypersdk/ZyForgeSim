"""Input adapters for ForgeSim."""

__all__ = [
    "ForgeBundle",
    "ForgeBundleAdapter",
    "ProfileLookupError",
    "ProfileRegistry",
    "TraceAdapter",
    "TraceRecord",
    "fabric_ai_job_to_job",
    "gpu_count_from_spec",
    "resolve_tenant",
]


def __getattr__(name: str):
    if name in ("ForgeBundle", "ForgeBundleAdapter"):
        from forgesim.adapters.bundle import ForgeBundle, ForgeBundleAdapter

        return {"ForgeBundle": ForgeBundle, "ForgeBundleAdapter": ForgeBundleAdapter}[name]
    if name in ("TraceAdapter", "TraceRecord"):
        from forgesim.adapters.trace import TraceAdapter, TraceRecord

        return {"TraceAdapter": TraceAdapter, "TraceRecord": TraceRecord}[name]
    if name in ("ProfileLookupError", "ProfileRegistry"):
        from forgesim.adapters.profiles import ProfileLookupError, ProfileRegistry

        return {"ProfileLookupError": ProfileLookupError, "ProfileRegistry": ProfileRegistry}[name]
    if name in ("fabric_ai_job_to_job", "gpu_count_from_spec", "resolve_tenant"):
        from forgesim.adapters.crd import fabric_ai_job_to_job, gpu_count_from_spec, resolve_tenant

        return {
            "fabric_ai_job_to_job": fabric_ai_job_to_job,
            "gpu_count_from_spec": gpu_count_from_spec,
            "resolve_tenant": resolve_tenant,
        }[name]
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
