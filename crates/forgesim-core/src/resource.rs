use crate::cluster::Cluster;
use crate::error::{SimError, SimResult};
use crate::mig::MigProfileRegistry;
use crate::mig::reconfigure_gpu;
use crate::models::{Job, Placement};

#[derive(Debug)]
pub struct ResourceManager {
    pub mig_registry: Option<MigProfileRegistry>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self { mig_registry: None }
    }

    pub fn with_mig(registry: MigProfileRegistry) -> Self {
        Self {
            mig_registry: Some(registry),
        }
    }

    pub fn can_place(&self, cluster: &Cluster, job: &Job) -> bool {
        if job.is_mig_job() {
            return self.can_place_mig(cluster, job);
        }
        self.can_place_whole_gpu(cluster, job)
    }

    pub fn allocate(
        &self,
        cluster: &mut Cluster,
        job: &Job,
        start_time: f64,
    ) -> SimResult<Placement> {
        if job.is_mig_job() {
            return self.allocate_mig(cluster, job, start_time);
        }
        self.allocate_whole_gpu(cluster, job, start_time)
    }

    fn can_place_whole_gpu(&self, cluster: &Cluster, job: &Job) -> bool {
        if job.gpu_count == 0 {
            return false;
        }
        let free: Vec<_> = cluster
            .all_gpus()
            .filter(|g| g.is_whole_gpu_free())
            .collect();
        if free.len() < job.gpu_count as usize {
            return false;
        }
        if job.gpu_memory_gb > 0.0 {
            let eligible = free
                .iter()
                .filter(|g| g.memory_gb >= job.gpu_memory_gb)
                .count();
            if eligible < job.gpu_count as usize {
                return false;
            }
        }
        true
    }

    fn allocate_whole_gpu(
        &self,
        cluster: &Cluster,
        job: &Job,
        start_time: f64,
    ) -> SimResult<Placement> {
        if !self.can_place_whole_gpu(cluster, job) {
            return Err(SimError::InsufficientGpus {
                need: job.gpu_count,
                available: cluster.free_gpu_count() as u32,
            });
        }

        let mut selected = Vec::new();
        for gpu in cluster.all_gpus() {
            if !gpu.is_whole_gpu_free() {
                continue;
            }
            if job.gpu_memory_gb > 0.0 && gpu.memory_gb < job.gpu_memory_gb {
                continue;
            }
            selected.push(gpu.id.clone());
            if selected.len() == job.gpu_count as usize {
                break;
            }
        }

        if selected.len() != job.gpu_count as usize {
            return Err(SimError::InsufficientGpus {
                need: job.gpu_count,
                available: cluster.free_gpu_count() as u32,
            });
        }

        Ok(Placement {
            job_id: job.id.clone(),
            gpu_ids: selected,
            start_time,
        })
    }

    fn can_place_mig(&self, cluster: &Cluster, job: &Job) -> bool {
        let Some(registry) = &self.mig_registry else {
            return false;
        };
        let profile = match job.mig_profile_name() {
            Some(p) => p,
            None => return false,
        };
        if registry.profile(profile).is_err() {
            return false;
        }
        let needed = job.mig_slices_needed();
        let mut available = 0u32;
        for gpu in cluster.all_gpus() {
            if !gpu.mig_capable {
                continue;
            }
            available += gpu.free_mig_slice_count(profile);
            if gpu.is_fully_idle() {
                if let Ok(spec) = registry.profile(profile) {
                    if gpu.slices.is_empty()
                        || gpu.active_mig_profile.as_deref() != Some(profile)
                    {
                        available += spec.max_per_gpu;
                    }
                }
            }
        }
        available >= needed
    }

    fn allocate_mig(
        &self,
        cluster: &mut Cluster,
        job: &Job,
        start_time: f64,
    ) -> SimResult<Placement> {
        let registry = self.mig_registry.as_ref().ok_or_else(|| {
            SimError::Config("MIG job submitted but no MIG profile registry configured".into())
        })?;
        let profile = job
            .mig_profile_name()
            .ok_or_else(|| SimError::Config("MIG job missing mig_profile".into()))?;
        registry.profile(profile)?;
        let needed = job.mig_slices_needed();
        let mut selected = Vec::new();
        let mut reconfig_delay = 0.0;

        for gpu in cluster.all_gpus() {
            for slice in &gpu.slices {
                if slice.profile == profile && slice.is_free() {
                    selected.push(slice.id.clone());
                    if selected.len() == needed as usize {
                        break;
                    }
                }
            }
            if selected.len() == needed as usize {
                break;
            }
        }

        if selected.len() < needed as usize {
            let gpu_id = cluster
                .all_gpus()
                .find(|g| {
                    g.mig_capable
                        && g.is_fully_idle()
                        && g.free_mig_slice_count(profile) < needed
                })
                .map(|g| g.id.clone());

            if let Some(gpu_id) = gpu_id {
                reconfigure_gpu(
                    cluster
                        .all_gpus_mut()
                        .find(|g| g.id == gpu_id)
                        .expect("gpu must exist"),
                    profile,
                    needed,
                    registry,
                )?;
                cluster.mig_reconfigs += 1;
                reconfig_delay = registry.reconfig_seconds;
                if let Some(gpu) = cluster.all_gpus().find(|g| g.id == gpu_id) {
                    for slice in &gpu.slices {
                        if slice.profile == profile && slice.is_free() {
                            selected.push(slice.id.clone());
                            if selected.len() == needed as usize {
                                break;
                            }
                        }
                    }
                }
            }
        }

        if selected.len() != needed as usize {
            return Err(SimError::InsufficientGpus {
                need: needed,
                available: selected.len() as u32,
            });
        }

        Ok(Placement {
            job_id: job.id.clone(),
            gpu_ids: selected,
            start_time: start_time + reconfig_delay,
        })
    }

    pub fn release(&self, _cluster: &mut Cluster, _job: &Job) {
        // GPU release handled by Cluster::finish_job
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::Cluster;
    use crate::mig::{MigHardwareConfig, MigProfileRegistry, MigProfileSpec};
    use crate::models::{Gpu, Node};
    use std::collections::HashMap;

    fn mig_registry() -> MigProfileRegistry {
        MigProfileRegistry::from_config(MigHardwareConfig {
            hardware: "H100_80GB".into(),
            reconfig_seconds: 30.0,
            profiles: HashMap::from([(
                "1g.10gb".into(),
                MigProfileSpec {
                    memory_gb: 10.0,
                    max_per_gpu: 7,
                },
            )]),
        })
    }

    fn mig_gpu_cluster() -> Cluster {
        Cluster::new(vec![Node {
            id: "node-0".into(),
            gpus: vec![Gpu {
                id: "gpu-0".into(),
                node_id: "node-0".into(),
                profile: "H100_80GB".into(),
                memory_gb: 80.0,
                nvlink_group: None,
                running_job_id: None,
                mig_capable: true,
                active_mig_profile: None,
                slices: Vec::new(),
            }],
        }])
    }

    #[test]
    fn no_partial_allocation() {
        let cluster = Cluster::new(vec![Node {
            id: "n0".into(),
            gpus: vec![
                Gpu {
                    id: "g0".into(),
                    node_id: "n0".into(),
                    profile: "H100".into(),
                    memory_gb: 80.0,
                    nvlink_group: None,
                    running_job_id: None,
                    mig_capable: false,
                    active_mig_profile: None,
                    slices: Vec::new(),
                },
                Gpu {
                    id: "g1".into(),
                    node_id: "n0".into(),
                    profile: "H100".into(),
                    memory_gb: 80.0,
                    nvlink_group: None,
                    running_job_id: None,
                    mig_capable: false,
                    active_mig_profile: None,
                    slices: Vec::new(),
                },
            ],
        }]);
        let rm = ResourceManager::new();
        let job = Job::new("j1", "big", 0.0, 10.0, 2);
        assert!(rm.can_place(&cluster, &job));
        let p = rm.allocate(&mut cluster.clone(), &job, 0.0).unwrap();
        assert_eq!(p.gpu_ids.len(), 2);
    }

    #[test]
    fn rejects_when_not_enough_gpus() {
        let cluster = Cluster::new(vec![Node {
            id: "n0".into(),
            gpus: vec![Gpu {
                id: "g0".into(),
                node_id: "n0".into(),
                profile: "H100".into(),
                memory_gb: 80.0,
                nvlink_group: None,
                running_job_id: None,
                mig_capable: false,
                active_mig_profile: None,
                slices: Vec::new(),
            }],
        }]);
        let rm = ResourceManager::new();
        let job = Job::new("j1", "big", 0.0, 10.0, 3);
        assert!(!rm.can_place(&cluster, &job));
    }

    #[test]
    fn mig_job_reconfigures_and_allocates_slices() {
        let mut cluster = mig_gpu_cluster();
        let rm = ResourceManager::with_mig(mig_registry());
        let mut job = Job::new("j1", "infer", 0.0, 10.0, 1);
        job.mig_profile = Some("1g.10gb".into());
        job.mig_count = Some(2);

        let placement = rm.allocate(&mut cluster, &job, 0.0).unwrap();
        assert_eq!(placement.gpu_ids.len(), 2);
        assert_eq!(placement.start_time, 30.0);
        assert_eq!(cluster.mig_reconfigs, 1);
        assert_eq!(cluster.all_gpus().next().unwrap().slices.len(), 2);
    }

    #[test]
    fn second_mig_job_reuses_existing_slices() {
        let mut cluster = mig_gpu_cluster();
        let rm = ResourceManager::with_mig(mig_registry());
        let mut job_a = Job::new("j1", "a", 0.0, 10.0, 1);
        job_a.mig_profile = Some("1g.10gb".into());
        job_a.mig_count = Some(2);
        let p1 = rm.allocate(&mut cluster, &job_a, 0.0).unwrap();
        cluster.start_job(job_a, &p1.gpu_ids, p1.start_time);

        let mut job_b = Job::new("j2", "b", 1.0, 5.0, 1);
        job_b.mig_profile = Some("1g.10gb".into());
        job_b.mig_count = Some(1);
        assert!(!rm.can_place(&cluster, &job_b));

        cluster.finish_job("j1", 40.0);
        let p2 = rm.allocate(&mut cluster, &job_b, 40.0).unwrap();
        assert_eq!(p2.gpu_ids.len(), 1);
        assert_eq!(p2.start_time, 40.0);
        assert_eq!(cluster.mig_reconfigs, 1);
    }
}
