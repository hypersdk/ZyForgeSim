//! Integration tests: full simulation pipelines (config → engine → metrics).

use std::path::PathBuf;

use forgesim_config::{
    load_forge_bundle, run_forge_bundle, run_simulation, run_trace_file, trace_diff_to_json,
    TraceDiffReport,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn integration_synthetic_m1_workload_completes() {
    let config = repo_root().join("configs/clusters/small_h100.yaml");
    if !config.exists() {
        return;
    }
    let metrics = run_simulation(&config).expect("synthetic M1 simulation");
    assert_eq!(metrics.jobs_completed, metrics.jobs_total);
    assert!(metrics.jobs_total >= 8);
    assert!(metrics.makespan > 0.0);
    assert!(metrics.gpu_utilization > 0.0);
}

#[test]
fn integration_priority_scheduler_prefers_high_priority_job() {
    let fifo_config = repo_root().join("configs/clusters/priority_fifo.yaml");
    let priority_config = repo_root().join("configs/clusters/priority_scheduler.yaml");
    if !fifo_config.exists() || !priority_config.exists() {
        return;
    }
    // Same workload (a filler job occupies the only GPU while a low- and a
    // high-priority job both arrive and queue up behind it), run under each
    // scheduler policy. Total work done is identical either way, so
    // makespan doesn't distinguish them — but mean_wait_time does: the
    // priority scheduler runs the high-priority job first once the GPU
    // frees up, so on average jobs wait less than under strict FIFO, which
    // runs the earlier-arriving low-priority job first.
    let fifo_metrics = run_simulation(&fifo_config).expect("fifo simulation");
    let priority_metrics = run_simulation(&priority_config).expect("priority simulation");

    assert_eq!(fifo_metrics.jobs_completed, fifo_metrics.jobs_total);
    assert_eq!(priority_metrics.jobs_completed, priority_metrics.jobs_total);
    assert_eq!(fifo_metrics.makespan, priority_metrics.makespan);
    assert!(priority_metrics.mean_wait_time < fifo_metrics.mean_wait_time);
}

#[test]
fn integration_forge_bundle_fifo_simulation() {
    let bundle = repo_root().join("tests/fixtures/forge");
    if !bundle.exists() {
        return;
    }
    let metrics = run_forge_bundle(
        &bundle,
        &repo_root().join("configs/profiles"),
        &repo_root().join("configs/gpu_type_registry.yaml"),
        &repo_root().join("configs/hardware"),
        &repo_root().join("configs/mig"),
        "fifo",
    )
    .expect("forge bundle simulation");
    assert_eq!(metrics.jobs_completed, metrics.jobs_total);
    assert!(metrics.jobs_total >= 3);
}

#[test]
fn integration_forge_bundle_gang_and_mig_fields() {
    let bundle = repo_root().join("tests/fixtures/forge");
    if !bundle.exists() {
        return;
    }
    let loaded = load_forge_bundle(
        &bundle,
        &repo_root().join("configs/profiles"),
        &repo_root().join("configs/gpu_type_registry.yaml"),
        &repo_root().join("configs/hardware"),
    )
    .expect("load forge bundle");

    let gang = loaded
        .jobs
        .iter()
        .find(|j| j.name == "gpt-distributed-training")
        .expect("gang job");
    assert_eq!(gang.gpu_count, 32);
    assert!(gang.gang_enabled);

    let mig = loaded
        .jobs
        .iter()
        .find(|j| j.name == "mig-inference")
        .expect("mig job");
    assert_eq!(mig.gpu_count, 2);
    assert_eq!(mig.mig_profile.as_deref(), Some("1g.10gb"));
    assert!(loaded.cluster.all_gpus().any(|g| g.mig_capable));
}

#[test]
fn integration_forge_bundle_quota_delays_second_job() {
    let bundle = repo_root().join("tests/fixtures/forge_quota");
    if !bundle.exists() {
        return;
    }
    // Cluster has 4 GPUs and both 2-GPU jobs could run concurrently, but
    // the tenant's FabricQuota caps it at 2 GPUs — job B must wait for
    // job A to finish and free the quota before it can start.
    let metrics = run_forge_bundle(
        &bundle,
        &repo_root().join("configs/profiles"),
        &repo_root().join("configs/gpu_type_registry.yaml"),
        &repo_root().join("configs/hardware"),
        &repo_root().join("configs/mig"),
        "fifo",
    )
    .expect("forge bundle simulation");
    assert_eq!(metrics.jobs_completed, 2);
    // Without quota enforcement both jobs run in parallel and makespan
    // equals one job's runtime (604800s); quota enforcement serializes
    // them, doubling it.
    assert_eq!(metrics.makespan, 1_209_600.0);
    assert!(metrics.mean_wait_time > 0.0);
}

#[test]
fn integration_trace_replay_matches_fifo_oracle() {
    let trace = repo_root().join("tests/fixtures/traces/fifo_match.jsonl");
    let cluster_config = repo_root().join("configs/clusters/single_gpu.yaml");
    if !trace.exists() || !cluster_config.exists() {
        return;
    }
    let cluster = forgesim_config::load_cluster_from_config(&cluster_config).expect("load cluster");
    let result = run_trace_file(&trace, cluster, "fifo").expect("trace replay");
    assert_eq!(result.report.differing_placements, 0);
    assert_eq!(result.report.matching_placements, 2);
}

#[test]
fn integration_trace_diff_report_serializes() {
    let trace = repo_root().join("tests/fixtures/traces/fifo_match.jsonl");
    let cluster_config = repo_root().join("configs/clusters/single_gpu.yaml");
    if !trace.exists() || !cluster_config.exists() {
        return;
    }
    let cluster = forgesim_config::load_cluster_from_config(&cluster_config).expect("load cluster");
    let result = run_trace_file(&trace, cluster, "fifo").expect("trace replay");
    let json = trace_diff_to_json(&result.report);
    assert!(json.contains("\"matching_placements\": 2"));
    let parsed: TraceDiffReport = serde_json::from_str(&json).expect("parse diff json");
    assert_eq!(parsed.scheduler, "fifo");
}

#[test]
fn integration_mig_workload_tracks_reconfigs() {
    let config = repo_root().join("configs/clusters/mig_single.yaml");
    if !config.exists() {
        return;
    }
    let metrics = run_simulation(&config).expect("mig simulation");
    assert_eq!(metrics.jobs_completed, 2);
    assert_eq!(metrics.jobs_total, 2);
    assert!(metrics.mig_reconfigs >= 1);
    assert!(metrics.makespan >= 90.0);
}
