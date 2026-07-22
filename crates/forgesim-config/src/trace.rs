//! Scheduler event trace replay and oracle comparison.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use forgesim_core::cluster::Cluster;
use forgesim_core::engine::SimulationEngine;
use forgesim_core::models::Job;
use forgesim_metrics::SimulationMetrics;
use forgesim_scheduler::FifoScheduler;
use serde::{Deserialize, Serialize};

use crate::{
    build_cluster, load_hardware_profiles, load_simulation_config, resolve_path, ConfigError,
    ConfigResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "PascalCase")]
pub enum TraceEvent {
    JobSubmitted {
        timestamp: f64,
        job: String,
        gpu_count: u32,
        runtime: f64,
        #[serde(default)]
        gpu_memory_gb: f64,
        #[serde(default)]
        priority: u32,
    },
    JobScheduled {
        timestamp: f64,
        job: String,
        node: String,
        gpus: Vec<GpuRef>,
    },
    JobCompleted {
        timestamp: f64,
        job: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GpuRef {
    Id(String),
    Index(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OraclePlacement {
    pub job_id: String,
    pub timestamp: f64,
    pub node: String,
    pub gpu_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulatedPlacement {
    pub job_id: String,
    pub start_time: f64,
    pub gpu_ids: Vec<String>,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlacementDiff {
    pub job_id: String,
    pub placement_match: bool,
    pub schedule_time_match: bool,
    pub oracle: OraclePlacement,
    pub simulated: SimulatedPlacement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDiffReport {
    pub scheduler: String,
    pub jobs_total: usize,
    pub oracle_schedules: usize,
    pub matching_placements: usize,
    pub differing_placements: usize,
    pub diffs: Vec<PlacementDiff>,
    pub simulation_metrics: SimulationMetrics,
}

#[derive(Debug, Clone)]
pub struct TraceReplayResult {
    pub report: TraceDiffReport,
}

pub fn load_trace(path: &Path) -> ConfigResult<Vec<TraceEvent>> {
    let content = fs::read_to_string(path)?;
    let mut events = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let event: TraceEvent = serde_json::from_str(trimmed).map_err(|e| {
            ConfigError::Invalid(format!(
                "invalid trace JSON at {}:{}: {e}",
                path.display(),
                line_no + 1
            ))
        })?;
        events.push(event);
    }
    if events.is_empty() {
        return Err(ConfigError::Invalid(format!(
            "trace file {} contains no events",
            path.display()
        )));
    }
    Ok(events)
}

pub fn load_cluster_from_config(config_path: &Path) -> ConfigResult<Cluster> {
    let config = load_simulation_config(config_path)?;
    let base = config_path.parent().unwrap_or_else(|| Path::new("."));
    let hw_dir = resolve_path(base, &config.hardware_profiles_dir);
    let profiles = load_hardware_profiles(&hw_dir)?;
    build_cluster(&config.cluster, &profiles)
}

fn normalize_gpu_refs(node: &str, gpus: &[GpuRef], cluster: &Cluster) -> ConfigResult<Vec<String>> {
    let mut ids = Vec::new();
    for gpu_ref in gpus {
        match gpu_ref {
            GpuRef::Id(id) => {
                if cluster.gpu(id).is_none() {
                    return Err(ConfigError::Invalid(format!(
                        "unknown gpu id '{id}' in trace"
                    )));
                }
                ids.push(id.clone());
            }
            GpuRef::Index(index) => {
                let id = format!("{node}-gpu-{index}");
                if cluster.gpu(&id).is_none() {
                    return Err(ConfigError::Invalid(format!(
                        "unknown gpu index {index} on node '{node}'"
                    )));
                }
                ids.push(id);
            }
        }
    }
    Ok(ids)
}

pub fn jobs_from_trace(events: &[TraceEvent]) -> ConfigResult<Vec<Job>> {
    let mut jobs = Vec::new();
    for event in events {
        if let TraceEvent::JobSubmitted {
            timestamp,
            job,
            gpu_count,
            runtime,
            gpu_memory_gb,
            priority,
        } = event
        {
            jobs.push(Job {
                id: job.clone(),
                name: job.clone(),
                arrival_time: *timestamp,
                runtime: *runtime,
                gpu_count: *gpu_count,
                gpu_memory_gb: *gpu_memory_gb,
                priority: *priority,
                ..Job::new(job, job, *timestamp, *runtime, *gpu_count)
            });
        }
    }
    if jobs.is_empty() {
        return Err(ConfigError::Invalid(
            "trace must contain at least one JobSubmitted event".into(),
        ));
    }
    Ok(jobs)
}

pub fn oracle_placements_from_trace(
    events: &[TraceEvent],
    cluster: &Cluster,
) -> ConfigResult<Vec<OraclePlacement>> {
    let mut placements = Vec::new();
    for event in events {
        if let TraceEvent::JobScheduled {
            timestamp,
            job,
            node,
            gpus,
        } = event
        {
            let gpu_ids = normalize_gpu_refs(node, gpus, cluster)?;
            placements.push(OraclePlacement {
                job_id: job.clone(),
                timestamp: *timestamp,
                node: node.clone(),
                gpu_ids,
            });
        }
    }
    Ok(placements)
}

fn nodes_for_gpus(cluster: &Cluster, gpu_ids: &[String]) -> Vec<String> {
    let mut nodes: Vec<String> = gpu_ids
        .iter()
        .filter_map(|id| cluster.gpu(id).map(|g| g.node_id.clone()))
        .collect();
    nodes.sort();
    nodes.dedup();
    nodes
}

fn simulated_placements(cluster: &Cluster) -> HashMap<String, SimulatedPlacement> {
    let mut map = HashMap::new();
    for job in &cluster.finished_jobs {
        let gpu_ids = job.assigned_gpus.clone();
        map.insert(
            job.id.clone(),
            SimulatedPlacement {
                job_id: job.id.clone(),
                start_time: job.start_time.unwrap_or(0.0),
                nodes: nodes_for_gpus(cluster, &gpu_ids),
                gpu_ids,
            },
        );
    }
    map
}

fn placement_sets_match(a: &[String], b: &[String]) -> bool {
    let set_a: HashSet<_> = a.iter().collect();
    let set_b: HashSet<_> = b.iter().collect();
    set_a == set_b
}

pub fn compare_schedules(
    oracle: &[OraclePlacement],
    simulated: &HashMap<String, SimulatedPlacement>,
    scheduler: &str,
    metrics: SimulationMetrics,
) -> TraceDiffReport {
    let mut diffs = Vec::new();
    let mut matching = 0usize;

    for entry in oracle {
        let simulated_entry =
            simulated
                .get(&entry.job_id)
                .cloned()
                .unwrap_or_else(|| SimulatedPlacement {
                    job_id: entry.job_id.clone(),
                    start_time: f64::NAN,
                    gpu_ids: Vec::new(),
                    nodes: Vec::new(),
                });

        let placement_match = placement_sets_match(&entry.gpu_ids, &simulated_entry.gpu_ids);
        let schedule_time_match = (entry.timestamp - simulated_entry.start_time).abs() < 1e-6;

        if placement_match && schedule_time_match {
            matching += 1;
        }

        diffs.push(PlacementDiff {
            job_id: entry.job_id.clone(),
            placement_match,
            schedule_time_match,
            oracle: entry.clone(),
            simulated: simulated_entry,
        });
    }

    TraceDiffReport {
        scheduler: scheduler.into(),
        jobs_total: metrics.jobs_total,
        oracle_schedules: oracle.len(),
        matching_placements: matching,
        differing_placements: oracle.len().saturating_sub(matching),
        diffs,
        simulation_metrics: metrics,
    }
}

pub fn run_trace_replay(
    events: &[TraceEvent],
    cluster: Cluster,
    scheduler: &str,
) -> ConfigResult<TraceReplayResult> {
    let jobs = jobs_from_trace(events)?;
    let oracle = oracle_placements_from_trace(events, &cluster)?;
    let jobs_total = jobs.len();

    let mut engine = match scheduler {
        "fifo" => SimulationEngine::new(cluster, FifoScheduler),
        other => {
            return Err(ConfigError::Invalid(format!(
                "unsupported scheduler type '{other}' for trace replay"
            )));
        }
    };

    engine.submit_jobs(jobs);
    engine.run();

    let metrics = SimulationMetrics::from_cluster(&engine.cluster, jobs_total);
    let simulated = simulated_placements(&engine.cluster);
    let report = compare_schedules(&oracle, &simulated, scheduler, metrics);

    Ok(TraceReplayResult { report })
}

pub fn run_trace_file(
    trace_path: &Path,
    cluster: Cluster,
    scheduler: &str,
) -> ConfigResult<TraceReplayResult> {
    let events = load_trace(trace_path)?;
    run_trace_replay(&events, cluster, scheduler)
}

pub fn trace_diff_to_json(report: &TraceDiffReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
}

/// Parse a single trace line (used by Python adapter parity tests).
pub fn parse_trace_line(line: &str) -> ConfigResult<TraceEvent> {
    serde_json::from_str(line.trim()).map_err(|e| ConfigError::Invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgesim_core::models::{Gpu, Node};
    use std::path::PathBuf;

    fn one_gpu_cluster() -> Cluster {
        Cluster::new(vec![Node {
            id: "node-0".into(),
            gpus: vec![Gpu::new("gpu-0", "node-0", "H100_80GB", 80.0)],
        }])
    }

    fn two_gpu_cluster() -> Cluster {
        Cluster::new(vec![Node {
            id: "node-0".into(),
            gpus: vec![
                Gpu::new("gpu-0", "node-0", "H100_80GB", 80.0),
                Gpu::new("gpu-1", "node-0", "H100_80GB", 80.0),
            ],
        }])
    }

    #[test]
    fn parses_trace_events() {
        let line =
            r#"{"timestamp":0,"event":"JobSubmitted","job":"j1","gpu_count":1,"runtime":10}"#;
        let event = parse_trace_line(line).unwrap();
        assert!(matches!(
            event,
            TraceEvent::JobSubmitted {
                job,
                gpu_count: 1,
                ..
            } if job == "j1"
        ));
    }

    #[test]
    fn normalizes_gpu_indices() {
        let cluster = Cluster::new(vec![Node {
            id: "node-4".into(),
            gpus: (0..4)
                .map(|i| Gpu::new(format!("node-4-gpu-{i}"), "node-4", "H100_80GB", 80.0))
                .collect(),
        }]);
        let events = vec![TraceEvent::JobScheduled {
            timestamp: 18.0,
            job: "llama70b".into(),
            node: "node-4".into(),
            gpus: vec![
                GpuRef::Index(0),
                GpuRef::Index(1),
                GpuRef::Index(2),
                GpuRef::Index(3),
            ],
        }];
        let oracle = oracle_placements_from_trace(&events, &cluster).unwrap();
        assert_eq!(oracle[0].gpu_ids.len(), 4);
        assert_eq!(oracle[0].gpu_ids[0], "node-4-gpu-0");
    }

    #[test]
    fn fifo_trace_replay_matches_oracle() {
        let events = vec![
            TraceEvent::JobSubmitted {
                timestamp: 0.0,
                job: "j1".into(),
                gpu_count: 1,
                runtime: 100.0,
                gpu_memory_gb: 0.0,
                priority: 0,
            },
            TraceEvent::JobSubmitted {
                timestamp: 5.0,
                job: "j2".into(),
                gpu_count: 1,
                runtime: 50.0,
                gpu_memory_gb: 0.0,
                priority: 0,
            },
            TraceEvent::JobScheduled {
                timestamp: 0.0,
                job: "j1".into(),
                node: "node-0".into(),
                gpus: vec![GpuRef::Id("gpu-0".into())],
            },
            TraceEvent::JobScheduled {
                timestamp: 100.0,
                job: "j2".into(),
                node: "node-0".into(),
                gpus: vec![GpuRef::Id("gpu-0".into())],
            },
        ];

        let result = run_trace_replay(&events, one_gpu_cluster(), "fifo").unwrap();
        assert_eq!(result.report.matching_placements, 2);
        assert_eq!(result.report.differing_placements, 0);
    }

    #[test]
    fn detects_scheduler_divergence() {
        let events = vec![
            TraceEvent::JobSubmitted {
                timestamp: 0.0,
                job: "j1".into(),
                gpu_count: 1,
                runtime: 100.0,
                gpu_memory_gb: 0.0,
                priority: 0,
            },
            TraceEvent::JobSubmitted {
                timestamp: 0.0,
                job: "j2".into(),
                gpu_count: 1,
                runtime: 50.0,
                gpu_memory_gb: 0.0,
                priority: 0,
            },
            TraceEvent::JobScheduled {
                timestamp: 0.0,
                job: "j2".into(),
                node: "node-0".into(),
                gpus: vec![GpuRef::Id("gpu-1".into())],
            },
            TraceEvent::JobScheduled {
                timestamp: 50.0,
                job: "j1".into(),
                node: "node-0".into(),
                gpus: vec![GpuRef::Id("gpu-0".into())],
            },
        ];

        let result = run_trace_replay(&events, two_gpu_cluster(), "fifo").unwrap();
        assert_eq!(result.report.differing_placements, 2);
        assert!(result.report.diffs.iter().all(|d| !d.placement_match));
    }

    #[test]
    fn loads_fixture_trace_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/traces/fifo_match.jsonl");
        if !path.exists() {
            return;
        }
        let cluster = load_cluster_from_config(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../configs/clusters/single_gpu.yaml"),
        )
        .unwrap();
        let result = run_trace_file(&path, cluster, "fifo").unwrap();
        assert_eq!(result.report.differing_placements, 0);
    }
}
