//! CLI integration tests: invoke the forge-sim binary end-to-end.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn forge_sim() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_forge-sim"));
    cmd.current_dir(repo_root());
    cmd
}

#[test]
fn cli_run_internal_config_emits_metrics() {
    let config = repo_root().join("configs/clusters/small_h100.yaml");
    if !config.exists() {
        return;
    }
    let output = forge_sim()
        .args([
            "run",
            "--config",
            config.to_str().expect("utf8 path"),
            "--output",
            "outputs/test_metrics_cli.json",
        ])
        .output()
        .expect("spawn forge-sim");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jobs completed"));
    assert!(repo_root().join("outputs/test_metrics_cli.json").exists());
}

#[test]
fn cli_run_forge_bundle_completes() {
    let bundle = repo_root().join("tests/fixtures/forge");
    if !bundle.exists() {
        return;
    }
    let output = forge_sim()
        .args([
            "run",
            "--forge-bundle",
            bundle.to_str().expect("utf8 path"),
            "--profiles-dir",
            "configs/profiles",
        ])
        .output()
        .expect("spawn forge-sim");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jobs completed"));
}

#[test]
fn cli_replay_trace_reports_zero_diffs() {
    let trace = repo_root().join("tests/fixtures/traces/fifo_match.jsonl");
    let config = repo_root().join("configs/clusters/single_gpu.yaml");
    if !trace.exists() || !config.exists() {
        return;
    }
    let output = forge_sim()
        .args([
            "replay",
            "--trace",
            trace.to_str().expect("utf8 path"),
            "--config",
            config.to_str().expect("utf8 path"),
            "--output",
            "outputs/test_trace_diff_cli.json",
        ])
        .output()
        .expect("spawn forge-sim replay");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("matching:"));
    assert!(stdout.contains("2/2"));
}

#[test]
fn cli_run_mig_config_reports_reconfigs() {
    let config = repo_root().join("configs/clusters/mig_single.yaml");
    if !config.exists() {
        return;
    }
    let output = forge_sim()
        .args(["run", "--config", config.to_str().expect("utf8 path")])
        .output()
        .expect("spawn forge-sim");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mig reconfigs:"));
}

#[test]
fn cli_run_requires_config_or_bundle() {
    let output = forge_sim().args(["run"]).output().expect("spawn forge-sim");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--config") || stderr.contains("--forge-bundle"));
}
