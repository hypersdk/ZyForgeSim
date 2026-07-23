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
                run_generation: 0,
            });
            self.pending_arrivals.push(job);
        }
    }

    pub fn run(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.cluster.clock = event.time;
            match event.kind {
                EventKind::JobArrival => self.handle_arrival(&event.job_id),
                EventKind::JobComplete => self.handle_complete(&event.job_id, event.run_generation),
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

    fn handle_complete(&mut self, job_id: &str, run_generation: u32) {
        // A job preempted and restarted since this event was scheduled has
        // moved on to a later generation — this event is stale, ignore it
        // rather than finishing a run that isn't the current one.
        match self.cluster.running_jobs.get(job_id) {
            Some(job) if job.run_generation == run_generation => {}
            _ => return,
        }
        self.cluster.finish_job(job_id, self.cluster.clock);
        self.try_schedule();
    }

    fn try_schedule(&mut self) {
        let placements = self
            .scheduler
            .schedule(&mut self.cluster, &self.resource_manager);
        for placement in placements {
            if let Some(mut job) = self.take_waiting_job(&placement.job_id) {
                job.run_generation += 1;
                let duration = job.duration_remaining();
                let run_generation = job.run_generation;
                self.cluster
                    .start_job(job.clone(), &placement.gpu_ids, placement.start_time);
                self.event_queue.push(Event {
                    time: placement.start_time + duration,
                    kind: EventKind::JobComplete,
                    job_id: job.id,
                    run_generation,
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
        fn schedule(&mut self, cluster: &mut Cluster, rm: &ResourceManager) -> Vec<Placement> {
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

    /// Evicts "low" to make room for "high" whenever "high" doesn't
    /// otherwise fit — a minimal stand-in for a preempting scheduler, used
    /// to exercise the engine's stale-JobComplete handling directly.
    struct PreemptingTestScheduler;

    impl Scheduler for PreemptingTestScheduler {
        fn schedule(&mut self, cluster: &mut Cluster, rm: &ResourceManager) -> Vec<Placement> {
            cluster.sort_waiting_by_arrival();
            let waiting = cluster.waiting_queue.to_vec();
            let mut placements = Vec::new();

            for job in waiting {
                if place(cluster, rm, &job, &mut placements) {
                    continue;
                }
                if job.id != "high" {
                    continue;
                }
                let Some(victim) = cluster.evict_job("low") else {
                    continue;
                };
                if place(cluster, rm, &job, &mut placements) {
                    let mut victim = victim;
                    victim.requeue_after_preemption(cluster.clock);
                    cluster.waiting_queue.push(victim);
                } else {
                    cluster.resume_evicted_job(victim);
                }
            }

            for placement in &placements {
                for resource_id in &placement.gpu_ids {
                    cluster.mark_resource_free(resource_id);
                }
            }
            placements
        }
    }

    fn place(
        cluster: &mut Cluster,
        rm: &ResourceManager,
        job: &Job,
        placements: &mut Vec<Placement>,
    ) -> bool {
        if !rm.can_place(cluster, job) {
            return false;
        }
        let Ok(placement) = rm.allocate(cluster, job, cluster.clock) else {
            return false;
        };
        for resource_id in &placement.gpu_ids {
            cluster.mark_resource_busy(resource_id, &job.id);
        }
        placements.push(placement);
        true
    }

    #[test]
    fn stale_job_complete_from_before_a_preemption_does_not_finish_the_resumed_run() {
        let mut engine = SimulationEngine::new(cluster(), PreemptingTestScheduler);
        // "low" starts at t=0 for 100s (JobComplete scheduled for t=100).
        // "high" arrives at t=5, evicts "low" (which has run 5s, 95s
        // left), and runs for 20s (finishes at t=25). At t=25, "low"
        // resumes for its remaining 95s (finishes at t=120). The stale
        // JobComplete(low) event from t=100 must not fire early.
        engine.submit_jobs(vec![
            Job::new("low", "low", 0.0, 100.0, 1),
            Job::new("high", "high", 5.0, 20.0, 1),
        ]);
        engine.run();

        assert_eq!(engine.cluster.finished_jobs.len(), 2);
        let low = engine
            .cluster
            .finished_jobs
            .iter()
            .find(|j| j.id == "low")
            .expect("low finished");
        assert_eq!(low.finish_time, Some(120.0));
        assert_eq!(low.preemption_count, 1);
        let high = engine
            .cluster
            .finished_jobs
            .iter()
            .find(|j| j.id == "high")
            .expect("high finished");
        assert_eq!(high.finish_time, Some(25.0));
    }
}
