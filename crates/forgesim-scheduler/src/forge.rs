//! Forge scheduler: priority ordering with preemption. Quotas, gang spread,
//! and topology locality are enforced by `ResourceManager`, not here.

pub use crate::preemptive::PreemptivePriorityScheduler as ForgeScheduler;
