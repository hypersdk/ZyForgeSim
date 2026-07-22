use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    #[default]
    Pending,
    Waiting,
    Running,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub arrival_time: f64,
    pub runtime: f64,
    pub gpu_count: u32,
    pub gpu_memory_gb: f64,
    pub priority: u32,
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub network_bw_gbps: Option<f64>,
    #[serde(default)]
    pub gpu_type: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub gang_enabled: bool,
    #[serde(default)]
    pub gang_size_nodes: Option<u32>,
    #[serde(default)]
    pub mig_profile: Option<String>,
    #[serde(default)]
    pub mig_count: Option<u32>,
    #[serde(default)]
    pub state: JobState,
    #[serde(default)]
    pub start_time: Option<f64>,
    #[serde(default)]
    pub finish_time: Option<f64>,
    #[serde(default, skip_serializing)]
    pub assigned_gpus: Vec<String>,
}

impl Job {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        arrival_time: f64,
        runtime: f64,
        gpu_count: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arrival_time,
            runtime,
            gpu_count,
            gpu_memory_gb: 0.0,
            priority: 0,
            tenant: None,
            network_bw_gbps: None,
            gpu_type: None,
            namespace: None,
            gang_enabled: false,
            gang_size_nodes: None,
            mig_profile: None,
            mig_count: None,
            state: JobState::Pending,
            start_time: None,
            finish_time: None,
            assigned_gpus: Vec::new(),
        }
    }

    pub fn wait_time(&self) -> f64 {
        match (self.start_time, self.state) {
            (Some(start), _) => (start - self.arrival_time).max(0.0),
            _ => 0.0,
        }
    }

    pub fn is_mig_job(&self) -> bool {
        self.mig_profile.is_some()
    }

    pub fn mig_slices_needed(&self) -> u32 {
        if self.is_mig_job() {
            self.mig_count.unwrap_or(1).max(1)
        } else {
            self.gpu_count
        }
    }

    pub fn mig_profile_name(&self) -> Option<&str> {
        self.mig_profile.as_deref()
    }
}

#[cfg(test)]
mod job_tests {
    use super::Job;

    #[test]
    fn mig_slices_needed_defaults_to_one() {
        let mut job = Job::new("j1", "infer", 0.0, 10.0, 8);
        job.mig_profile = Some("1g.10gb".into());
        assert_eq!(job.mig_slices_needed(), 1);
    }

    #[test]
    fn mig_slices_needed_uses_count() {
        let mut job = Job::new("j1", "infer", 0.0, 10.0, 8);
        job.mig_profile = Some("1g.10gb".into());
        job.mig_count = Some(3);
        assert_eq!(job.mig_slices_needed(), 3);
    }

    #[test]
    fn non_mig_job_uses_gpu_count() {
        let job = Job::new("j1", "train", 0.0, 100.0, 4);
        assert!(!job.is_mig_job());
        assert_eq!(job.mig_slices_needed(), 4);
    }

    #[test]
    fn wait_time_is_zero_before_start() {
        let job = Job::new("j1", "a", 5.0, 10.0, 1);
        assert_eq!(job.wait_time(), 0.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSlice {
    pub id: String,
    pub profile: String,
    pub memory_gb: f64,
    #[serde(default, skip_serializing)]
    pub running_job_id: Option<String>,
}

impl MigSlice {
    pub fn is_free(&self) -> bool {
        self.running_job_id.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gpu {
    pub id: String,
    pub node_id: String,
    pub profile: String,
    pub memory_gb: f64,
    #[serde(default)]
    pub nvlink_group: Option<u32>,
    #[serde(default, skip_serializing)]
    pub running_job_id: Option<String>,
    #[serde(default)]
    pub mig_capable: bool,
    #[serde(default, skip_serializing)]
    pub active_mig_profile: Option<String>,
    #[serde(default, skip_serializing)]
    pub slices: Vec<MigSlice>,
}

impl Gpu {
    pub fn new(
        id: impl Into<String>,
        node_id: impl Into<String>,
        profile: impl Into<String>,
        memory_gb: f64,
    ) -> Self {
        Self {
            id: id.into(),
            node_id: node_id.into(),
            profile: profile.into(),
            memory_gb,
            nvlink_group: None,
            running_job_id: None,
            mig_capable: false,
            active_mig_profile: None,
            slices: Vec::new(),
        }
    }

    pub fn is_free(&self) -> bool {
        if !self.slices.is_empty() {
            return false;
        }
        self.running_job_id.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub gpus: Vec<Gpu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub job_id: String,
    pub gpu_ids: Vec<String>,
    pub start_time: f64,
}
