use forgesim_core::cluster::Cluster;
use forgesim_core::models::JobState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTimelineRecord {
    pub job_id: String,
    pub name: String,
    pub arrival_time: f64,
    pub start_time: Option<f64>,
    pub finish_time: Option<f64>,
    pub runtime: f64,
    pub gpu_count: u32,
    pub assigned_gpus: Vec<String>,
    pub priority: u32,
    pub tenant: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsTimeline {
    pub makespan: f64,
    pub gpu_count: usize,
    pub jobs: Vec<JobTimelineRecord>,
}

impl JobsTimeline {
    pub fn from_cluster(cluster: &Cluster) -> Self {
        let mut jobs: Vec<JobTimelineRecord> = cluster
            .finished_jobs
            .iter()
            .map(job_to_timeline_record)
            .collect();
        for job in cluster.running_jobs.values() {
            jobs.push(job_to_timeline_record(job));
        }
        for job in &cluster.waiting_queue {
            jobs.push(job_to_timeline_record(job));
        }
        jobs.sort_by(|a, b| {
            a.arrival_time
                .partial_cmp(&b.arrival_time)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.job_id.cmp(&b.job_id))
        });

        let makespan = jobs
            .iter()
            .filter_map(|j| j.finish_time)
            .fold(0.0_f64, f64::max);

        Self {
            makespan,
            gpu_count: cluster.gpu_count(),
            jobs,
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

fn job_to_timeline_record(job: &forgesim_core::models::Job) -> JobTimelineRecord {
    JobTimelineRecord {
        job_id: job.id.clone(),
        name: job.name.clone(),
        arrival_time: job.arrival_time,
        start_time: job.start_time,
        finish_time: job.finish_time,
        runtime: job.runtime,
        gpu_count: job.gpu_count,
        assigned_gpus: job.assigned_gpus.clone(),
        priority: job.priority,
        tenant: job.tenant.clone(),
        state: format!("{:?}", job.state).to_lowercase(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub makespan: f64,
    pub mean_wait_time: f64,
    pub gpu_utilization: f64,
    pub jobs_completed: usize,
    pub jobs_total: usize,
    pub queue_max_length: usize,
    #[serde(default)]
    pub mig_reconfigs: u32,
    #[serde(default)]
    pub preemptions: u32,
    #[serde(default)]
    pub topology_penalties: u32,
    #[serde(default)]
    pub topology_runtime_inflation: f64,
    #[serde(default)]
    pub jobs_failed: usize,
}

impl SimulationMetrics {
    pub fn from_cluster(cluster: &Cluster, jobs_total: usize) -> Self {
        let finished = &cluster.finished_jobs;
        let finished_success: Vec<_> = finished
            .iter()
            .filter(|j| j.state == JobState::Finished)
            .collect();
        let jobs_failed = finished
            .iter()
            .filter(|j| j.state == JobState::Failed)
            .count();

        let makespan = finished
            .iter()
            .filter_map(|j| j.finish_time)
            .fold(0.0_f64, f64::max);

        let mean_wait_time = if finished_success.is_empty() {
            0.0
        } else {
            finished_success.iter().map(|j| j.wait_time()).sum::<f64>()
                / finished_success.len() as f64
        };

        let gpu_seconds_busy: f64 = finished_success
            .iter()
            .map(|j| {
                match (j.start_time, j.finish_time) {
                    (Some(s), Some(f)) => (f - s).max(0.0) * j.gpu_count as f64,
                    _ => j.runtime * j.gpu_count as f64,
                }
            })
            .sum();
        let gpu_count = cluster.gpu_count().max(1) as f64;
        let gpu_utilization = if makespan > 0.0 {
            (gpu_seconds_busy / (makespan * gpu_count)).min(1.0)
        } else {
            0.0
        };

        Self {
            makespan,
            mean_wait_time,
            gpu_utilization,
            jobs_completed: finished_success.len(),
            jobs_total,
            queue_max_length: 0,
            mig_reconfigs: cluster.mig_reconfigs,
            preemptions: cluster.total_preemptions,
            topology_penalties: cluster.topology_penalties,
            topology_runtime_inflation: cluster.topology_runtime_inflation,
            jobs_failed,
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgesim_core::models::{Gpu, Job, JobState, Node};

    #[test]
    fn computes_makespan_and_utilization() {
        let mut cluster = Cluster::new(vec![Node {
            id: "n0".into(),
            gpus: vec![Gpu::new("g0", "n0", "H100", 80.0)],
        }]);
        let mut job = Job::new("j1", "a", 0.0, 10.0, 1);
        job.state = JobState::Finished;
        job.start_time = Some(0.0);
        job.finish_time = Some(10.0);
        cluster.finished_jobs.push(job);
        cluster.clock = 10.0;

        let m = SimulationMetrics::from_cluster(&cluster, 1);
        assert_eq!(m.makespan, 10.0);
        assert!((m.gpu_utilization - 1.0).abs() < 1e-6);
    }
}
