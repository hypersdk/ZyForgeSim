use forgesim_core::cluster::Cluster;
use forgesim_core::engine::Scheduler;
use forgesim_core::models::Placement;
use forgesim_core::resource::ResourceManager;

/// First-in-first-out scheduler: earliest arrival first, all-or-nothing GPU placement.
#[derive(Debug, Default, Clone)]
pub struct FifoScheduler;

impl Scheduler for FifoScheduler {
    fn schedule(
        &mut self,
        cluster: &mut Cluster,
        resource_manager: &ResourceManager,
    ) -> Vec<Placement> {
        cluster.sort_waiting_by_arrival();
        let waiting: Vec<_> = cluster.waiting_queue.to_vec();
        let mut placements = Vec::new();

        for job in waiting {
            if !resource_manager.can_place(cluster, &job) {
                continue;
            }
            match resource_manager.allocate(cluster, &job, cluster.clock) {
                Ok(placement) => {
                    for resource_id in &placement.gpu_ids {
                        if let Some(slice) = cluster.slice_mut(resource_id) {
                            slice.running_job_id = Some(job.id.clone());
                        } else if let Some(gpu) = cluster.gpu_mut(resource_id) {
                            gpu.running_job_id = Some(job.id.clone());
                        }
                    }
                    placements.push(placement);
                }
                Err(_) => continue,
            }
        }

        for placement in &placements {
            for resource_id in &placement.gpu_ids {
                if let Some(slice) = cluster.slice_mut(resource_id) {
                    slice.running_job_id = None;
                } else if let Some(gpu) = cluster.gpu_mut(resource_id) {
                    gpu.running_job_id = None;
                }
            }
        }

        placements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgesim_core::models::{Gpu, Job, Node};

    #[test]
    fn fifo_schedules_earlier_job_first() {
        let mut cluster = Cluster::new(vec![Node {
            id: "n0".into(),
            gpus: vec![Gpu::new("g0", "n0", "H100", 80.0)],
        }]);
        cluster.clock = 0.0;
        cluster.enqueue_job(Job::new("j2", "late", 5.0, 10.0, 1));
        cluster.enqueue_job(Job::new("j1", "early", 0.0, 10.0, 1));

        let mut sched = FifoScheduler;
        let rm = ResourceManager::new();
        let placements = sched.schedule(&mut cluster, &rm);

        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].job_id, "j1");
    }
}
