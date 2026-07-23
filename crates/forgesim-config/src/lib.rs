mod forge_bundle;
mod mig;
mod trace;

pub use forge_bundle::{
    load_forge_bundle, load_gpu_type_registry, load_model_profiles, parse_fabric_ai_job,
    run_forge_bundle, run_forge_bundle_report, ForgeBundle, GpuTypeRegistry, ModelProfile,
};
pub use mig::{
    load_mig_registry, load_mig_registry_for_hardware, resolve_mig_registry_for_cluster,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
pub use trace::{
    compare_schedules, jobs_from_trace, load_cluster_from_config, load_trace,
    oracle_placements_from_trace, parse_trace_line, run_trace_file, run_trace_replay,
    trace_diff_to_json, validate_job_gang_config, GpuRef, OraclePlacement, PlacementDiff,
    SimulatedPlacement, TraceDiffReport, TraceEvent, TraceReplayResult,
};

use forgesim_core::cluster::Cluster;
use forgesim_core::engine::{Scheduler, SimulationEngine};
use forgesim_core::models::{Gpu, Job, Node};
use forgesim_core::resource::{GpuSelectionPolicy, ResourceManager};
use forgesim_core::rl::RlSession;
use forgesim_core::topology::TopologyGraph;
use forgesim_metrics::{JobsTimeline, SimulationMetrics};
use forgesim_scheduler::{
    BestFitScheduler, FifoScheduler, ForgeScheduler, PriorityScheduler,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub metrics: SimulationMetrics,
    pub timeline: JobsTimeline,
    #[serde(default)]
    pub decisions: Vec<forgesim_core::SchedulerDecision>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("config error: {0}")]
    Invalid(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Clone, Deserialize)]
pub struct HardwareProfile {
    pub name: String,
    pub memory_gb: f64,
    #[serde(default)]
    pub sm: Option<u32>,
    #[serde(default)]
    pub mig_profiles: Vec<String>,
    #[serde(default)]
    pub nvlink_bw_gbs: Option<f64>,
    #[serde(default)]
    pub pcie_bw_gbs: Option<f64>,
    #[serde(default)]
    pub mig: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GpuSpec {
    pub id: String,
    pub profile: String,
    #[serde(default)]
    pub nvlink_group: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub gpus: Vec<GpuSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClusterConfig {
    pub nodes: Vec<NodeSpec>,
    #[serde(default)]
    pub tenant_quotas: HashMap<String, u32>,
    /// Synthetic NVLink layout: `nvlink_pairs` (default), `full_mesh`, or `pcie_only`.
    #[serde(default = "default_topology_template")]
    pub topology_template: String,
}

fn default_topology_template() -> String {
    "nvlink_pairs".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    #[serde(default = "default_scheduler")]
    pub r#type: String,
}

fn default_scheduler() -> String {
    "fifo".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkloadRef {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimulationConfig {
    pub cluster: ClusterConfig,
    pub scheduler: SchedulerConfig,
    pub workload: WorkloadRef,
    #[serde(default = "default_hardware_dir")]
    pub hardware_profiles_dir: String,
    #[serde(default = "default_mig_profiles_dir")]
    pub mig_profiles_dir: String,
}

fn default_hardware_dir() -> String {
    "configs/hardware".into()
}

fn default_mig_profiles_dir() -> String {
    "../mig".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkloadJobSpec {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub arrival_time: f64,
    pub runtime: f64,
    pub gpu_count: u32,
    #[serde(default)]
    pub gpu_memory_gb: f64,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub network_bw_gbps: Option<f64>,
    #[serde(default)]
    pub gpu_type: Option<String>,
    #[serde(default)]
    pub mig_profile: Option<String>,
    #[serde(default)]
    pub mig_count: Option<u32>,
    #[serde(default)]
    pub gang_enabled: bool,
    #[serde(default)]
    pub gang_size_nodes: Option<u32>,
    #[serde(default)]
    pub gang_timeout_secs: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkloadConfig {
    pub jobs: Vec<WorkloadJobSpec>,
}

pub fn load_hardware_profiles(dir: &Path) -> ConfigResult<HashMap<String, HardwareProfile>> {
    let mut profiles = HashMap::new();
    if !dir.exists() {
        return Ok(profiles);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let profile: HardwareProfile = serde_yaml::from_str(&content)?;
        profiles.insert(profile.name.clone(), profile);
    }
    Ok(profiles)
}

pub fn load_simulation_config(path: &Path) -> ConfigResult<SimulationConfig> {
    let content = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&content)?)
}

pub fn load_workload(path: &Path) -> ConfigResult<Vec<Job>> {
    let content = fs::read_to_string(path)?;
    let workload: WorkloadConfig = serde_yaml::from_str(&content)?;
    let mut jobs = Vec::new();
    for j in workload.jobs {
        let name = j.name.unwrap_or_else(|| "job".into());
        let job = Job {
            id: j.id.clone(),
            name: name.clone(),
            arrival_time: j.arrival_time,
            runtime: j.runtime,
            gpu_count: j.gpu_count,
            gpu_memory_gb: j.gpu_memory_gb,
            priority: j.priority,
            tenant: j.tenant,
            gpu_type: j.gpu_type,
            network_bw_gbps: j.network_bw_gbps,
            mig_profile: j.mig_profile,
            mig_count: j.mig_count,
            gang_enabled: j.gang_enabled,
            gang_size_nodes: j.gang_size_nodes,
            gang_timeout_secs: j.gang_timeout_secs,
            ..Job::new(j.id, name, j.arrival_time, j.runtime, j.gpu_count)
        };
        validate_job_gang_config(&job)?;
        jobs.push(job);
    }
    Ok(jobs)
}

pub fn build_cluster(
    cluster_cfg: &ClusterConfig,
    profiles: &HashMap<String, HardwareProfile>,
) -> ConfigResult<Cluster> {
    let mut nodes = Vec::new();
    for node_spec in &cluster_cfg.nodes {
        let mut gpus = Vec::new();
        for gpu_spec in &node_spec.gpus {
            let profile = profiles.get(&gpu_spec.profile).ok_or_else(|| {
                ConfigError::Invalid(format!("unknown hardware profile '{}'", gpu_spec.profile))
            })?;
            let mut gpu = Gpu::new(
                gpu_spec.id.clone(),
                node_spec.id.clone(),
                gpu_spec.profile.clone(),
                profile.memory_gb,
            );
            gpu.nvlink_group = gpu_spec.nvlink_group.or_else(|| {
                apply_topology_template(&cluster_cfg.topology_template, gpu_spec, node_spec)
            });
            gpu.mig_capable = profile.mig;
            gpus.push(gpu);
        }
        nodes.push(Node {
            id: node_spec.id.clone(),
            gpus,
        });
    }
    let mut nvlink_bw: f64 = 900.0;
    let mut pcie_bw: f64 = 64.0;
    for node_spec in &cluster_cfg.nodes {
        for gpu_spec in &node_spec.gpus {
            if let Some(profile) = profiles.get(&gpu_spec.profile) {
                if let Some(v) = profile.nvlink_bw_gbs {
                    nvlink_bw = nvlink_bw.max(v);
                }
                if let Some(v) = profile.pcie_bw_gbs {
                    pcie_bw = pcie_bw.max(v);
                }
            }
        }
    }
    let mut cluster = Cluster::new(nodes);
    cluster.topology = TopologyGraph::from_profile_bandwidths(nvlink_bw, pcie_bw);
    cluster.tenant_quotas = cluster_cfg.tenant_quotas.clone();
    Ok(cluster)
}

fn apply_topology_template(
    template: &str,
    gpu_spec: &GpuSpec,
    node_spec: &NodeSpec,
) -> Option<u32> {
    let gpu_index = node_spec
        .gpus
        .iter()
        .position(|g| g.id == gpu_spec.id)?;
    match template {
        "full_mesh" => Some(0),
        "pcie_only" => Some(gpu_index as u32),
        _ => Some((gpu_index / 2) as u32),
    }
}

pub fn resolve_path(base: &Path, relative: &str) -> PathBuf {
    let p = PathBuf::from(relative);
    if p.is_absolute() {
        p
    } else {
        base.join(p)
    }
}

pub fn run_simulation(config_path: &Path) -> ConfigResult<SimulationMetrics> {
    Ok(run_simulation_report(config_path)?.metrics)
}

pub fn run_simulation_report(config_path: &Path) -> ConfigResult<SimulationReport> {
    let config = load_simulation_config(config_path)?;
    let base = config_path.parent().unwrap_or_else(|| Path::new("."));

    let hw_dir = resolve_path(base, &config.hardware_profiles_dir);
    let profiles = load_hardware_profiles(&hw_dir)?;

    let workload_path = resolve_path(base, &config.workload.path);
    let jobs = load_workload(&workload_path)?;
    let jobs_total = jobs.len();

    let cluster = build_cluster(&config.cluster, &profiles)?;
    let hardware_names: Vec<String> = config
        .cluster
        .nodes
        .iter()
        .flat_map(|n| n.gpus.iter().map(|g| g.profile.clone()))
        .collect();
    let any_mig_capable = cluster.all_gpus().any(|g| g.mig_capable);
    let any_mig_job = jobs.iter().any(|j| j.is_mig_job());
    let mig_dir = resolve_path(base, &config.mig_profiles_dir);
    let mig_registry = if any_mig_job || any_mig_capable {
        resolve_mig_registry_for_cluster(&mig_dir, &hardware_names, any_mig_capable)?
    } else {
        None
    };
    if any_mig_job && mig_registry.is_none() {
        return Err(ConfigError::Invalid(
            "workload contains MIG jobs but no MIG profile registry is configured".into(),
        ));
    }

    let resource_manager = build_resource_manager(mig_registry, &config.scheduler.r#type);

    let (cluster, metrics) = run_to_completion_with_policy(
        cluster,
        &config.scheduler.r#type,
        resource_manager,
        jobs,
        jobs_total,
    )?;

    Ok(SimulationReport {
        metrics,
        timeline: JobsTimeline::from_cluster(&cluster),
        decisions: cluster.decision_log.clone(),
    })
}

pub fn load_rl_session(config_path: &Path) -> ConfigResult<RlSession> {
    let config = load_simulation_config(config_path)?;
    let base = config_path.parent().unwrap_or_else(|| Path::new("."));

    let hw_dir = resolve_path(base, &config.hardware_profiles_dir);
    let profiles = load_hardware_profiles(&hw_dir)?;

    let workload_path = resolve_path(base, &config.workload.path);
    let jobs = load_workload(&workload_path)?;

    let cluster = build_cluster(&config.cluster, &profiles)?;
    let hardware_names: Vec<String> = config
        .cluster
        .nodes
        .iter()
        .flat_map(|n| n.gpus.iter().map(|g| g.profile.clone()))
        .collect();
    let any_mig_capable = cluster.all_gpus().any(|g| g.mig_capable);
    let any_mig_job = jobs.iter().any(|j| j.is_mig_job());
    let mig_dir = resolve_path(base, &config.mig_profiles_dir);
    let mig_registry = if any_mig_job || any_mig_capable {
        resolve_mig_registry_for_cluster(&mig_dir, &hardware_names, any_mig_capable)?
    } else {
        None
    };
    if any_mig_job && mig_registry.is_none() {
        return Err(ConfigError::Invalid(
            "workload contains MIG jobs but no MIG profile registry is configured".into(),
        ));
    }

    let resource_manager = build_resource_manager(mig_registry, &config.scheduler.r#type);

    Ok(RlSession::new(
        cluster,
        resource_manager,
        jobs,
        forgesim_core::DEFAULT_OBS_TOP_K,
    ))
}

pub fn build_resource_manager(
    mig_registry: Option<forgesim_core::mig::MigProfileRegistry>,
    scheduler: &str,
) -> ResourceManager {
    let rm = match mig_registry {
        Some(registry) => ResourceManager::with_mig(registry),
        None => ResourceManager::new(),
    };
    if scheduler == "bestfit" {
        rm.with_gpu_selection(GpuSelectionPolicy::BestFit)
    } else {
        rm
    }
}

fn run_to_completion_with_policy(
    cluster: Cluster,
    scheduler: &str,
    resource_manager: ResourceManager,
    jobs: Vec<Job>,
    jobs_total: usize,
) -> ConfigResult<(Cluster, SimulationMetrics)> {
    Ok(match scheduler {
        "fifo" => run_to_completion(cluster, FifoScheduler, resource_manager, jobs, jobs_total),
        "priority" => run_to_completion(
            cluster,
            PriorityScheduler,
            resource_manager,
            jobs,
            jobs_total,
        ),
        "preemptive" | "forge" => run_to_completion(
            cluster,
            ForgeScheduler::default(),
            resource_manager,
            jobs,
            jobs_total,
        ),
        "bestfit" => run_to_completion(
            cluster,
            BestFitScheduler,
            resource_manager,
            jobs,
            jobs_total,
        ),
        other => {
            return Err(ConfigError::Invalid(format!(
                "unsupported scheduler type '{other}'"
            )));
        }
    })
}

fn run_to_completion<S: Scheduler>(
    cluster: Cluster,
    scheduler: S,
    resource_manager: ResourceManager,
    jobs: Vec<Job>,
    jobs_total: usize,
) -> (Cluster, SimulationMetrics) {
    let mut engine = SimulationEngine::with_resource_manager(cluster, scheduler, resource_manager);
    engine.submit_jobs(jobs);
    engine.run();
    (
        engine.cluster.clone(),
        SimulationMetrics::from_cluster(&engine.cluster, jobs_total),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_sample_workload() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/workloads/synthetic_m1.yaml");
        if root.exists() {
            let jobs = load_workload(&root).unwrap();
            assert!(!jobs.is_empty());
        }
    }

    #[test]
    fn runs_mig_workload() {
        let config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/clusters/mig_single.yaml");
        if !config.exists() {
            return;
        }
        let metrics = run_simulation(&config).unwrap();
        assert_eq!(metrics.jobs_completed, metrics.jobs_total);
        assert!(metrics.mig_reconfigs >= 1);
    }

    #[test]
    fn load_workload_with_mig_fields() {
        let workload =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/workloads/mig_m4.yaml");
        if !workload.exists() {
            return;
        }
        let jobs = load_workload(&workload).unwrap();
        let mig = jobs
            .iter()
            .find(|j| j.id == "mig-infer-a")
            .expect("mig job");
        assert_eq!(mig.mig_profile.as_deref(), Some("1g.10gb"));
        assert_eq!(mig.mig_count, Some(2));
        assert!(mig.is_mig_job());
    }
}
