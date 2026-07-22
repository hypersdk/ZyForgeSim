use crate::cluster::Cluster;
use crate::events::{Event, EventKind, EventQueue};
use crate::models::{Job, JobState, Placement};
use crate::resource::ResourceManager;

pub trait Scheduler {
    fn schedule(
        &mut self,
        cluster: &mut Cluster,
        resource_manager: &ResourceManager,
    ) -> Vec<Placement>;
}

pub struct SimulationEngine<S: Scheduler> {
    pub cluster: Cluster,
    pub resource_manager: ResourceManager,
    pub scheduler: S,
    event_queue: EventQueue,
    pending_arrivals: Vec<Job>,
}

impl<S: Scheduler> SimulationEngine<S> {
    pub fn new(cluster: Cluster, scheduler: S) -> Self {
        Self::with_resource_manager(cluster, scheduler, ResourceManager::new())
    }

    pub fn with_resource_manager(
        cluster: Cluster,
        scheduler: S,
        resource_manager: ResourceManager,
    ) -> Self {
        Self {
            cluster,
            resource_manager,
            scheduler,
            event_queue: EventQueue::new(),
            pending_arrivals: Vec::new(),
        }
    }

    pub fn submit_jobs(&mut self, mut jobs: Vec<Job>) {
        jobs.sort_by(|a, b| {
            a.arrival_time
                .partial_cmp(&b.arrival_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for job in jobs {
            self.event_queue.push(Event {
                time: job.arrival_time,
                kind: EventKind::JobArrival,
                job_id: job.id.clone(),
            });
            self.pending_arrivals.push(job);
        }
    }

    pub fn run(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.cluster.clock = event.time;
            match event.kind {
                EventKind::JobArrival => self.handle_arrival(&event.job_id),
                EventKind::JobComplete => self.handle_complete(&event.job_id),
            }
        }
    }

    fn handle_arrival(&mut self, job_id: &str) {
        let idx = self
            .pending_arrivals
            .iter()
            .position(|j| j.id == job_id)
            .expect("arrival job must exist");
        let mut job = self.pending_arrivals.remove(idx);
        job.state = JobState::Waiting;
        self.cluster.enqueue_job(job);
        self.try_schedule();
    }

    fn handle_complete(&mut self, job_id: &str) {
        self.cluster.finish_job(job_id, self.cluster.clock);
        self.try_schedule();
    }

    fn try_schedule(&mut self) {
        let placements = self
            .scheduler
            .schedule(&mut self.cluster, &self.resource_manager);
        for placement in placements {
            if let Some(job) = self.take_waiting_job(&placement.job_id) {
                self.cluster
                    .start_job(job.clone(), &placement.gpu_ids, placement.start_time);
                self.event_queue.push(Event {
                    time: placement.start_time + job.runtime,
                    kind: EventKind::JobComplete,
                    job_id: job.id,
                });
            }
        }
    }

    fn take_waiting_job(&mut self, job_id: &str) -> Option<Job> {
        let idx = self
            .cluster
            .waiting_queue
            .iter()
            .position(|j| j.id == job_id)?;
        Some(self.cluster.waiting_queue.remove(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Gpu, Node};

    struct TestScheduler;

    impl Scheduler for TestScheduler {
        fn schedule(
            &mut self,
            cluster: &mut Cluster,
            rm: &ResourceManager,
        ) -> Vec<Placement> {
            cluster.sort_waiting_by_arrival();
            let mut placements = Vec::new();
            let waiting: Vec<_> = cluster.waiting_queue.iter().cloned().collect();
            for job in waiting {
                if rm.can_place(cluster, &job) {
                    if let Ok(p) = rm.allocate(cluster, &job, cluster.clock) {
                        placements.push(p);
                    }
                }
            }
            placements
        }
    }

    fn cluster() -> Cluster {
        Cluster::new(vec![Node {
            id: "n0".into(),
            gpus: vec![Gpu::new("g0", "n0", "H100", 80.0)],
        }])
    }

    #[test]
    fn runs_jobs_to_completion() {
        let mut engine = SimulationEngine::new(cluster(), TestScheduler);
        engine.submit_jobs(vec![
            Job::new("j1", "a", 0.0, 5.0, 1),
            Job::new("j2", "b", 2.0, 3.0, 1),
        ]);
        engine.run();
        assert_eq!(engine.cluster.finished_jobs.len(), 2);
    }
}
