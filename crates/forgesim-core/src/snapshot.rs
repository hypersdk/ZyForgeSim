use crate::cluster::Cluster;
use crate::models::Job;
use serde::{Deserialize, Serialize};

pub const DEFAULT_OBS_TOP_K: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSnapshot {
    pub id: String,
    pub name: String,
    pub arrival_time: f64,
    pub runtime: f64,
    pub gpu_count: u32,
    pub priority: u32,
    pub wait_proxy: f64,
    pub placeable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSnapshot {
    pub clock: f64,
    pub free_gpus: u32,
    pub waiting: u32,
    pub running: u32,
    pub finished: u32,
    pub top_jobs: Vec<JobSnapshot>,
}

pub fn job_wait_proxy(job: &Job, clock: f64) -> f64 {
    (clock - job.arrival_time).max(0.0)
}

impl ClusterSnapshot {
    pub fn from_cluster(
        cluster: &Cluster,
        top_k: usize,
        placeable_mask: &[bool],
    ) -> Self {
        let waiting: Vec<_> = cluster.waiting_queue.iter().collect();
        let mut top_jobs = Vec::new();
        for (idx, job) in waiting.iter().take(top_k).enumerate() {
            top_jobs.push(JobSnapshot {
                id: job.id.clone(),
                name: job.name.clone(),
                arrival_time: job.arrival_time,
                runtime: job.runtime,
                gpu_count: job.gpu_count,
                priority: job.priority,
                wait_proxy: job_wait_proxy(job, cluster.clock),
                placeable: placeable_mask.get(idx).copied().unwrap_or(false),
            });
        }

        Self {
            clock: cluster.clock,
            free_gpus: cluster.free_gpu_count() as u32,
            waiting: cluster.waiting_queue.len() as u32,
            running: cluster.running_jobs.len() as u32,
            finished: cluster.finished_jobs.len() as u32,
            top_jobs,
        }
    }

    pub fn to_feature_vector(&self) -> Vec<f32> {
        let mut features = vec![
            self.clock as f32,
            self.free_gpus as f32,
            self.waiting as f32,
            self.running as f32,
            self.finished as f32,
        ];
        for job in &self.top_jobs {
            features.push(job.arrival_time as f32);
            features.push(job.runtime as f32);
            features.push(job.gpu_count as f32);
            features.push(job.priority as f32);
            features.push(job.wait_proxy as f32);
            features.push(if job.placeable { 1.0 } else { 0.0 });
        }
        features
    }
}

pub fn obs_size(top_k: usize) -> usize {
    5 + top_k * 6
}
