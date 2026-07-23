use forgesim_core::cluster::Cluster;
use serde::{Deserialize, Serialize};

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
}

impl SimulationMetrics {
    pub fn from_cluster(cluster: &Cluster, jobs_total: usize) -> Self {
        let finished = &cluster.finished_jobs;
        let makespan = finished
            .iter()
            .filter_map(|j| j.finish_time)
            .fold(0.0_f64, f64::max);

        let mean_wait_time = if finished.is_empty() {
            0.0
        } else {
            finished.iter().map(|j| j.wait_time()).sum::<f64>() / finished.len() as f64
        };

        let gpu_seconds_busy: f64 = finished
            .iter()
            .map(|j| j.runtime * j.gpu_count as f64)
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
            jobs_completed: finished.len(),
            jobs_total,
            queue_max_length: 0,
            mig_reconfigs: cluster.mig_reconfigs,
            preemptions: cluster.total_preemptions,
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
