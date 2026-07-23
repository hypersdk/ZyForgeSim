//! Load normalized Forge export bundles (FabricAIJob, FabricGpuNode, FabricQuota).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use forgesim_core::cluster::Cluster;
use forgesim_core::models::{Gpu, Job, Node};
use forgesim_core::resource::ResourceManager;
use forgesim_metrics::{JobsTimeline, SimulationMetrics};
use forgesim_scheduler::{FifoScheduler, PreemptivePriorityScheduler, PriorityScheduler};
use serde::Deserialize;
use serde_yaml::Value;

use crate::{
    load_hardware_profiles, resolve_mig_registry_for_cluster, ConfigError, ConfigResult,
    HardwareProfile, SimulationReport,
};

const FORGE_API_GROUP: &str = "forge.ai/v1";

#[derive(Debug, Clone, Deserialize)]
pub struct GpuTypeRegistry {
    pub mappings: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelProfileEntry {
    pub runtime_seconds: f64,
    pub gpu_memory_gb: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelProfile {
    pub model: String,
    pub profiles: HashMap<String, ModelProfileEntry>,
}

#[derive(Debug, Clone)]
pub struct ForgeBundle {
    pub jobs: Vec<Job>,
    pub cluster: Cluster,
}

pub fn load_gpu_type_registry(path: &Path) -> ConfigResult<GpuTypeRegistry> {
    let content = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&content)?)
}

pub fn load_model_profiles(dir: &Path) -> ConfigResult<HashMap<String, ModelProfile>> {
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
        let profile: ModelProfile = serde_yaml::from_str(&content)?;
        profiles.insert(profile.model.clone(), profile);
    }
    Ok(profiles)
}

fn map_gpu_type(gpu_type: &str, registry: &GpuTypeRegistry) -> ConfigResult<String> {
    registry
        .mappings
        .get(gpu_type)
        .cloned()
        .ok_or_else(|| ConfigError::Invalid(format!("unknown gpuType '{gpu_type}'")))
}

fn yaml_documents(content: &str) -> ConfigResult<Vec<Value>> {
    let mut docs = Vec::new();
    for chunk in content.split("\n---") {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }
        let doc: Value = serde_yaml::from_str(trimmed)?;
        if doc.is_null() {
            continue;
        }
        // `kubectl get <resource> -A -o yaml` wraps multiple resources in a
        // single `kind: List` document with an `items:` array, rather than
        // `---`-separating them — exactly the export command documented in
        // docs/forge_input.md. Unwrap it so downstream kind/apiVersion
        // checks see the actual FabricAIJob/FabricGpuNode/FabricQuota docs.
        if kind_of(&doc) == Some("List") {
            if let Some(items) = doc.get("items").and_then(|i| i.as_sequence()) {
                docs.extend(items.iter().cloned());
            }
            continue;
        }
        docs.push(doc);
    }
    Ok(docs)
}

fn kind_of(doc: &Value) -> Option<&str> {
    doc.get("kind").and_then(|k| k.as_str())
}

fn api_version_of(doc: &Value) -> Option<&str> {
    doc.get("apiVersion").and_then(|k| k.as_str())
}

fn validate_forge_doc(doc: &Value) -> ConfigResult<()> {
    match api_version_of(doc) {
        Some(v) if v == FORGE_API_GROUP => Ok(()),
        Some(v) => Err(ConfigError::Invalid(format!(
            "unsupported apiVersion '{v}', expected '{FORGE_API_GROUP}'"
        ))),
        None => Err(ConfigError::Invalid("missing apiVersion".into())),
    }
}

fn collect_yaml_files(dir: &Path) -> ConfigResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "yaml" || ext == "yml" {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn parse_fabric_quotas(dir: &Path) -> ConfigResult<Vec<Value>> {
    let mut quotas = Vec::new();
    for path in collect_yaml_files(dir)? {
        let content = fs::read_to_string(&path)?;
        for doc in yaml_documents(&content)? {
            if kind_of(&doc) == Some("FabricQuota") {
                validate_forge_doc(&doc)?;
                quotas.push(doc);
            }
        }
    }
    Ok(quotas)
}

fn parse_tenant_quotas(quotas: &[Value]) -> HashMap<String, u32> {
    let mut result = HashMap::new();
    for quota in quotas {
        let Some(team) = quota
            .get("spec")
            .and_then(|s| s.get("team"))
            .and_then(|t| t.as_str())
        else {
            continue;
        };
        let Some(max_gpus) = quota
            .get("spec")
            .and_then(|s| s.get("gpuQuota"))
            .and_then(|q| q.get("maxGPUs"))
            .and_then(|m| m.as_i64())
        else {
            continue;
        };
        result.insert(team.to_string(), max_gpus as u32);
    }
    result
}

fn resolve_tenant(namespace: &str, quotas: &[Value]) -> Option<String> {
    for quota in quotas {
        let spec = quota.get("spec")?;
        let team = spec.get("team").and_then(|t| t.as_str())?;
        if let Some(namespaces) = spec.get("namespaces").and_then(|n| n.as_sequence()) {
            for ns in namespaces {
                if ns.as_str() == Some(namespace) {
                    return Some(team.to_string());
                }
            }
        } else {
            return Some(team.to_string());
        }
    }
    None
}

fn gpu_count_from_spec(spec: &Value) -> u32 {
    let distributed = spec.get("distributed");
    if distributed
        .and_then(|d| d.get("enabled"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false)
    {
        let nodes = distributed
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_i64())
            .unwrap_or(1) as u32;
        let gpn = distributed
            .and_then(|d| d.get("gpusPerNode"))
            .and_then(|n| n.as_i64())
            .unwrap_or(1) as u32;
        nodes * gpn
    } else {
        spec.get("gpus").and_then(|g| g.as_i64()).unwrap_or(1) as u32
    }
}

fn parse_arrival_time(meta: &Value) -> f64 {
    meta.get("creationTimestamp")
        .and_then(|t| t.as_str())
        .map(|_| 0.0)
        .unwrap_or(0.0)
}

fn lookup_runtime(
    model: &str,
    gpu_type: &str,
    profiles: &HashMap<String, ModelProfile>,
) -> ConfigResult<(f64, f64)> {
    let profile = profiles.get(model).ok_or_else(|| {
        ConfigError::Invalid(format!(
            "no calibrated profile for model '{model}' (required in profiles-dir)"
        ))
    })?;
    let entry = profile.profiles.get(gpu_type).ok_or_else(|| {
        ConfigError::Invalid(format!(
            "no calibrated profile for model '{model}' gpuType '{gpu_type}'"
        ))
    })?;
    Ok((entry.runtime_seconds, entry.gpu_memory_gb))
}

pub fn parse_fabric_ai_job(
    doc: &Value,
    quotas: &[Value],
    model_profiles: &HashMap<String, ModelProfile>,
    _gpu_registry: &GpuTypeRegistry,
) -> ConfigResult<Job> {
    validate_forge_doc(doc)?;
    if kind_of(doc) != Some("FabricAIJob") {
        return Err(ConfigError::Invalid("expected FabricAIJob".into()));
    }

    let meta = doc
        .get("metadata")
        .ok_or_else(|| ConfigError::Invalid("missing metadata".into()))?;
    let spec = doc
        .get("spec")
        .ok_or_else(|| ConfigError::Invalid("missing spec".into()))?;

    let name = meta
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    let namespace = meta
        .get("namespace")
        .and_then(|n| n.as_str())
        .unwrap_or("default")
        .to_string();

    let annotations = meta.get("annotations").and_then(|a| a.as_mapping());
    let gang_enabled = annotations
        .and_then(|a| a.get(Value::from("forge.ai/gang-schedule")))
        .and_then(|v| v.as_str())
        .map(|s| s == "true")
        .unwrap_or(false);
    let gang_size_nodes = annotations
        .and_then(|a| a.get(Value::from("forge.ai/gang-size")))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok());

    let gpu_type = spec
        .get("gpuType")
        .and_then(|g| g.as_str())
        .unwrap_or("any")
        .to_string();
    let model = spec
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or(name)
        .to_string();

    let (runtime, gpu_memory_gb) = lookup_runtime(&model, &gpu_type, model_profiles)?;

    let mig = spec.get("mig");
    let mig_profile = mig
        .and_then(|m| m.get("profile"))
        .and_then(|p| p.as_str())
        .map(String::from);
    let mig_count = mig
        .and_then(|m| m.get("count"))
        .and_then(|c| c.as_i64())
        .map(|c| c as u32);

    let network_bw = match spec.get("network").and_then(|n| n.as_str()) {
        Some("rdma") => Some(400.0),
        Some("sriov") => Some(200.0),
        _ => None,
    };

    let priority = spec.get("priority").and_then(|p| p.as_i64()).unwrap_or(0) as u32;

    let gpu_count = if mig_profile.is_some() {
        mig_count.unwrap_or(1)
    } else {
        gpu_count_from_spec(spec)
    };

    Ok(Job {
        id: format!("{namespace}/{name}"),
        name: name.to_string(),
        arrival_time: parse_arrival_time(meta),
        runtime,
        gpu_count,
        gpu_memory_gb,
        priority,
        tenant: resolve_tenant(&namespace, quotas),
        network_bw_gbps: network_bw,
        gpu_type: Some(gpu_type.clone()),
        namespace: Some(namespace),
        gang_enabled,
        gang_size_nodes,
        mig_profile,
        mig_count,
        ..Job::new("", "", 0.0, 0.0, 0)
    })
}

pub fn parse_fabric_gpu_nodes(
    dir: &Path,
    gpu_registry: &GpuTypeRegistry,
    hw_profiles: &HashMap<String, HardwareProfile>,
) -> ConfigResult<Cluster> {
    let mut nodes = Vec::new();
    for path in collect_yaml_files(dir)? {
        let content = fs::read_to_string(&path)?;
        for doc in yaml_documents(&content)? {
            if kind_of(&doc) != Some("FabricGpuNode") {
                continue;
            }
            validate_forge_doc(&doc)?;
            let spec = doc
                .get("spec")
                .ok_or_else(|| ConfigError::Invalid("missing spec".into()))?;
            let node_name = spec
                .get("nodeName")
                .and_then(|n| n.as_str())
                .ok_or_else(|| ConfigError::Invalid("FabricGpuNode missing nodeName".into()))?;
            let gpu_type = spec
                .get("gpuType")
                .and_then(|g| g.as_str())
                .unwrap_or("any");
            let gpu_count = spec.get("gpuCount").and_then(|g| g.as_i64()).unwrap_or(1) as u32;
            let memory_gb = spec
                .get("memoryGB")
                .and_then(|m| m.as_i64())
                .map(|m| m as f64)
                .or_else(|| {
                    map_gpu_type(gpu_type, gpu_registry)
                        .ok()
                        .and_then(|p| hw_profiles.get(&p).map(|hp| hp.memory_gb))
                })
                .unwrap_or(80.0);

            let profile_name = map_gpu_type(gpu_type, gpu_registry)?;

            let hw_profile = hw_profiles.get(&profile_name);
            let mut gpus = Vec::new();
            for i in 0..gpu_count {
                let mut gpu = Gpu::new(
                    format!("{node_name}-gpu-{i}"),
                    node_name.to_string(),
                    profile_name.clone(),
                    memory_gb,
                );
                gpu.nvlink_group = Some(i / 2);
                gpu.mig_capable = hw_profile.map(|p| p.mig).unwrap_or(false);
                gpus.push(gpu);
            }
            nodes.push(Node {
                id: node_name.to_string(),
                gpus,
            });
        }
    }
    if nodes.is_empty() {
        return Err(ConfigError::Invalid(
            "no FabricGpuNode documents found in cluster/".into(),
        ));
    }
    Ok(Cluster::new(nodes))
}

pub fn load_forge_bundle(
    bundle_dir: &Path,
    profiles_dir: &Path,
    gpu_registry_path: &Path,
    hardware_profiles_dir: &Path,
) -> ConfigResult<ForgeBundle> {
    let quotas_dir = bundle_dir.join("quotas");
    let jobs_dir = bundle_dir.join("jobs");
    let cluster_dir = bundle_dir.join("cluster");

    let quotas = parse_fabric_quotas(&quotas_dir)?;
    let model_profiles = load_model_profiles(profiles_dir)?;
    let gpu_registry = load_gpu_type_registry(gpu_registry_path)?;
    let hw_profiles = load_hardware_profiles(hardware_profiles_dir)?;

    let mut jobs = Vec::new();
    for path in collect_yaml_files(&jobs_dir)? {
        let content = fs::read_to_string(&path)?;
        for doc in yaml_documents(&content)? {
            if kind_of(&doc) == Some("FabricAIJob") {
                jobs.push(parse_fabric_ai_job(
                    &doc,
                    &quotas,
                    &model_profiles,
                    &gpu_registry,
                )?);
            }
        }
    }

    if jobs.is_empty() {
        return Err(ConfigError::Invalid(
            "no FabricAIJob documents in jobs/".into(),
        ));
    }

    let mut cluster = parse_fabric_gpu_nodes(&cluster_dir, &gpu_registry, &hw_profiles)?;
    cluster.tenant_quotas = parse_tenant_quotas(&quotas);

    Ok(ForgeBundle { jobs, cluster })
}

pub fn run_forge_bundle_report(
    bundle_dir: &Path,
    profiles_dir: &Path,
    gpu_registry_path: &Path,
    hardware_profiles_dir: &Path,
    mig_profiles_dir: &Path,
    scheduler: &str,
) -> ConfigResult<SimulationReport> {
    let bundle = load_forge_bundle(
        bundle_dir,
        profiles_dir,
        gpu_registry_path,
        hardware_profiles_dir,
    )?;
    let jobs_total = bundle.jobs.len();
    let hardware_names: Vec<String> = bundle
        .cluster
        .all_gpus()
        .map(|g| g.profile.clone())
        .collect();
    let any_mig_capable = bundle.cluster.all_gpus().any(|g| g.mig_capable);
    let any_mig_job = bundle.jobs.iter().any(|j| j.is_mig_job());
    let mig_registry = if any_mig_job || any_mig_capable {
        resolve_mig_registry_for_cluster(mig_profiles_dir, &hardware_names, any_mig_capable)?
    } else {
        None
    };
    if any_mig_job && mig_registry.is_none() {
        return Err(ConfigError::Invalid(
            "forge bundle contains MIG jobs but no MIG profile registry is configured".into(),
        ));
    }
    let resource_manager = match mig_registry {
        Some(registry) => ResourceManager::with_mig(registry),
        None => ResourceManager::new(),
    };
    let (cluster, metrics) = match scheduler {
        "fifo" => crate::run_to_completion(
            bundle.cluster,
            FifoScheduler,
            resource_manager,
            bundle.jobs,
            jobs_total,
        ),
        "priority" => crate::run_to_completion(
            bundle.cluster,
            PriorityScheduler,
            resource_manager,
            bundle.jobs,
            jobs_total,
        ),
        "preemptive" => crate::run_to_completion(
            bundle.cluster,
            PreemptivePriorityScheduler,
            resource_manager,
            bundle.jobs,
            jobs_total,
        ),
        other => {
            return Err(ConfigError::Invalid(format!(
                "unsupported scheduler type '{other}'"
            )));
        }
    };
    Ok(SimulationReport {
        metrics,
        timeline: JobsTimeline::from_cluster(&cluster),
    })
}

pub fn run_forge_bundle(
    bundle_dir: &Path,
    profiles_dir: &Path,
    gpu_registry_path: &Path,
    hardware_profiles_dir: &Path,
    mig_profiles_dir: &Path,
    scheduler: &str,
) -> ConfigResult<SimulationMetrics> {
    Ok(run_forge_bundle_report(
        bundle_dir,
        profiles_dir,
        gpu_registry_path,
        hardware_profiles_dir,
        mig_profiles_dir,
        scheduler,
    )?
    .metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn gpu_count_distributed() {
        let spec: Value = serde_yaml::from_str(
            r#"
            gpus: 8
            distributed:
              enabled: true
              nodes: 4
              gpusPerNode: 8
            "#,
        )
        .unwrap();
        assert_eq!(gpu_count_from_spec(&spec), 32);
    }

    #[test]
    fn gpu_count_non_distributed() {
        let spec: Value = serde_yaml::from_str("gpus: 4").unwrap();
        assert_eq!(gpu_count_from_spec(&spec), 4);
    }

    #[test]
    fn yaml_documents_unwraps_kubectl_list_output() {
        // `kubectl get fabricaijobs -A -o yaml` (the exact command
        // docs/forge_input.md tells users to run) wraps every matching
        // resource in a single `kind: List` document instead of
        // `---`-separating them.
        let content = r#"
apiVersion: v1
items:
- apiVersion: forge.ai/v1
  kind: FabricAIJob
  metadata:
    name: job-a
    namespace: default
  spec:
    model: llama-7b
    gpus: 4
    gpuType: A100
- apiVersion: forge.ai/v1
  kind: FabricAIJob
  metadata:
    name: job-b
    namespace: default
  spec:
    model: gpt-13b
    gpus: 2
    gpuType: H100
kind: List
metadata:
  resourceVersion: ""
"#;
        let docs = yaml_documents(content).expect("parse kubectl List output");
        assert_eq!(docs.len(), 2);
        assert_eq!(kind_of(&docs[0]), Some("FabricAIJob"));
        assert_eq!(
            docs[0]
                .get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str()),
            Some("job-a")
        );
        assert_eq!(api_version_of(&docs[1]), Some("forge.ai/v1"));
    }

    #[test]
    fn yaml_documents_still_handles_dash_separated_docs() {
        let content = "apiVersion: forge.ai/v1\nkind: FabricAIJob\nmetadata:\n  name: a\n---\napiVersion: forge.ai/v1\nkind: FabricAIJob\nmetadata:\n  name: b\n";
        let docs = yaml_documents(content).expect("parse multi-doc yaml");
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn yaml_documents_empty_list_yields_no_docs() {
        let content = "apiVersion: v1\nitems: []\nkind: List\n";
        let docs = yaml_documents(content).expect("parse empty list");
        assert!(docs.is_empty());
    }

    #[test]
    fn loads_fixture_forge_bundle() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/forge");
        if !root.exists() {
            return;
        }
        let profiles = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/profiles");
        let registry =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/gpu_type_registry.yaml");
        let hw = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/hardware");

        let bundle = load_forge_bundle(&root, &profiles, &registry, &hw).unwrap();
        let gang = bundle
            .jobs
            .iter()
            .find(|j| j.name == "gpt-distributed-training")
            .expect("gang job");
        assert_eq!(gang.gpu_count, 32);
        assert_eq!(gang.tenant.as_deref(), Some("ml-training"));

        let mig = bundle
            .jobs
            .iter()
            .find(|j| j.name == "mig-inference")
            .expect("mig job");
        assert_eq!(mig.gpu_count, 2);
        assert_eq!(mig.mig_profile.as_deref(), Some("1g.10gb"));
    }

    #[test]
    fn forge_bundle_e2e_simulation() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/forge");
        if !root.exists() {
            return;
        }
        let profiles = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/profiles");
        let registry =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/gpu_type_registry.yaml");
        let hw = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/hardware");
        let mig = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/mig");

        let metrics = run_forge_bundle(&root, &profiles, &registry, &hw, &mig, "fifo").unwrap();
        assert_eq!(metrics.jobs_completed, metrics.jobs_total);
        assert!(metrics.jobs_total >= 2);
    }
}
