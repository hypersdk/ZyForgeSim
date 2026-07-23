use std::cmp::Ordering;
use std::collections::HashMap;

use crate::models::{Gpu, Job, JobState, MigSlice, Node};

#[derive(Debug, Clone)]
pub struct Cluster {
    pub nodes: Vec<Node>,
    pub waiting_queue: Vec<Job>,
    pub running_jobs: HashMap<String, Job>,
    pub finished_jobs: Vec<Job>,
    pub clock: f64,
    pub mig_reconfigs: u32,
    /// Max GPUs a tenant may hold across running jobs, keyed by tenant name.
    /// Tenants with no entry are unrestricted.
    pub tenant_quotas: HashMap<String, u32>,
}

impl Cluster {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
            waiting_queue: Vec::new(),
            running_jobs: HashMap::new(),
            finished_jobs: Vec::new(),
            clock: 0.0,
            mig_reconfigs: 0,
            tenant_quotas: HashMap::new(),
        }
    }

    /// GPUs currently held by `tenant` across its running jobs.
    pub fn tenant_gpu_usage(&self, tenant: &str) -> u32 {
        self.running_jobs
            .values()
            .filter(|j| j.tenant.as_deref() == Some(tenant))
            .map(|j| j.gpu_count)
            .sum()
    }

    pub fn all_gpus(&self) -> impl Iterator<Item = &Gpu> {
        self.nodes.iter().flat_map(|n| n.gpus.iter())
    }

    pub fn all_gpus_mut(&mut self) -> impl Iterator<Item = &mut Gpu> {
        self.nodes.iter_mut().flat_map(|n| n.gpus.iter_mut())
    }

    pub fn gpu_count(&self) -> usize {
        self.nodes.iter().map(|n| n.gpus.len()).sum()
    }

    pub fn free_gpu_count(&self) -> usize {
        self.all_gpus().filter(|g| g.is_whole_gpu_free()).count()
    }

    pub fn enqueue_job(&mut self, job: Job) {
        self.waiting_queue.push(job);
    }

    pub fn start_job(&mut self, job: Job, placement_resource_ids: &[String], start_time: f64) {
        for resource_id in placement_resource_ids {
            self.mark_resource_busy(resource_id, &job.id);
        }

        let mut running = job;
        running.state = JobState::Running;
        running.start_time = Some(start_time);
        running.assigned_gpus = placement_resource_ids.to_vec();
        self.running_jobs.insert(running.id.clone(), running);
    }

    pub fn finish_job(&mut self, job_id: &str, finish_time: f64) -> Option<Job> {
        let mut job = self.running_jobs.remove(job_id)?;
        for resource_id in &job.assigned_gpus {
            self.mark_resource_free(resource_id);
        }
        job.state = JobState::Finished;
        job.finish_time = Some(finish_time);
        self.finished_jobs.push(job.clone());
        Some(job)
    }

    pub fn mark_resource_busy(&mut self, resource_id: &str, job_id: &str) {
        if let Some(slice) = self.slice_mut(resource_id) {
            slice.running_job_id = Some(job_id.to_string());
            return;
        }
        if let Some(gpu) = self.gpu_mut(resource_id) {
            gpu.running_job_id = Some(job_id.to_string());
        }
    }

    pub fn mark_resource_free(&mut self, resource_id: &str) {
        if let Some(slice) = self.slice_mut(resource_id) {
            slice.running_job_id = None;
            return;
        }
        if let Some(gpu) = self.gpu_mut(resource_id) {
            gpu.running_job_id = None;
        }
    }

    pub fn gpu(&self, resource_id: &str) -> Option<&Gpu> {
        if self.slice(resource_id).is_some() {
            return self
                .all_gpus()
                .find(|g| g.slices.iter().any(|s| s.id == resource_id));
        }
        self.all_gpus().find(|g| g.id == resource_id)
    }

    pub fn gpu_mut(&mut self, resource_id: &str) -> Option<&mut Gpu> {
        if self.contains_slice(resource_id) {
            return None;
        }
        self.all_gpus_mut().find(|g| g.id == resource_id)
    }

    pub fn slice(&self, slice_id: &str) -> Option<&MigSlice> {
        for gpu in self.all_gpus() {
            if let Some(slice) = gpu.slices.iter().find(|s| s.id == slice_id) {
                return Some(slice);
            }
        }
        None
    }

    pub fn slice_mut(&mut self, slice_id: &str) -> Option<&mut MigSlice> {
        for gpu in self.all_gpus_mut() {
            if let Some(slice) = gpu.slices.iter_mut().find(|s| s.id == slice_id) {
                return Some(slice);
            }
        }
        None
    }

    fn contains_slice(&self, slice_id: &str) -> bool {
        self.slice(slice_id).is_some()
    }

    pub fn sort_waiting_by_arrival(&mut self) {
        self.waiting_queue.sort_by(|a, b| {
            a.arrival_time
                .partial_cmp(&b.arrival_time)
                .unwrap_or(Ordering::Equal)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Gpu;

    fn sample_node() -> Node {
        Node {
            id: "node-0".into(),
            gpus: vec![
                Gpu {
                    id: "gpu-0".into(),
                    node_id: "node-0".into(),
                    profile: "H100_80GB".into(),
                    memory_gb: 80.0,
                    nvlink_group: Some(1),
                    running_job_id: None,
                    mig_capable: false,
                    active_mig_profile: None,
                    slices: Vec::new(),
                },
                Gpu {
                    id: "gpu-1".into(),
                    node_id: "node-0".into(),
                    profile: "H100_80GB".into(),
                    memory_gb: 80.0,
                    nvlink_group: Some(1),
                    running_job_id: None,
                    mig_capable: false,
                    active_mig_profile: None,
                    slices: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn tenant_gpu_usage_sums_running_jobs_for_tenant() {
        let mut cluster = Cluster::new(vec![sample_node()]);
        let mut a = Job::new("j1", "a", 0.0, 10.0, 1);
        a.tenant = Some("acme".into());
        let mut b = Job::new("j2", "b", 0.0, 10.0, 1);
        b.tenant = Some("other".into());
        cluster.start_job(a, &["gpu-0".into()], 0.0);
        cluster.start_job(b, &["gpu-1".into()], 0.0);

        assert_eq!(cluster.tenant_gpu_usage("acme"), 1);
        assert_eq!(cluster.tenant_gpu_usage("other"), 1);
        assert_eq!(cluster.tenant_gpu_usage("nobody"), 0);
    }

    #[test]
    fn finish_job_frees_gpus() {
        let mut cluster = Cluster::new(vec![sample_node()]);
        let job = Job::new("j1", "job-1", 0.0, 10.0, 1);
        cluster.start_job(job, &["gpu-0".into()], 0.0);
        assert_eq!(cluster.free_gpu_count(), 1);
        cluster.finish_job("j1", 10.0);
        assert_eq!(cluster.free_gpu_count(), 2);
    }

    #[test]
    fn finish_job_frees_mig_slices() {
        let mut gpu = sample_node().gpus.into_iter().next().unwrap();
        gpu.mig_capable = true;
        gpu.slices.push(MigSlice {
            id: "gpu-0-mig-0".into(),
            profile: "1g.10gb".into(),
            memory_gb: 10.0,
            running_job_id: None,
        });
        let mut cluster = Cluster::new(vec![Node {
            id: "node-0".into(),
            gpus: vec![gpu],
        }]);
        let job = Job::new("j1", "job-1", 0.0, 10.0, 1);
        cluster.start_job(job, &["gpu-0-mig-0".into()], 0.0);
        assert!(cluster
            .slice("gpu-0-mig-0")
            .unwrap()
            .running_job_id
            .is_some());
        cluster.finish_job("j1", 10.0);
        assert!(cluster
            .slice("gpu-0-mig-0")
            .unwrap()
            .running_job_id
            .is_none());
    }
}
