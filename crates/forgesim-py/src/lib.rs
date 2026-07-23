use forgesim_config::{load_rl_session, run_simulation};
use forgesim_core::rl::RlSession;
use forgesim_metrics::SimulationMetrics;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Clone)]
struct SimResult {
    #[pyo3(get)]
    makespan: f64,
    #[pyo3(get)]
    mean_wait_time: f64,
    #[pyo3(get)]
    gpu_utilization: f64,
    #[pyo3(get)]
    jobs_completed: usize,
    #[pyo3(get)]
    jobs_total: usize,
    #[pyo3(get)]
    mig_reconfigs: u32,
    #[pyo3(get)]
    preemptions: u32,
    #[pyo3(get)]
    topology_penalties: u32,
}

impl From<SimulationMetrics> for SimResult {
    fn from(m: SimulationMetrics) -> Self {
        Self {
            makespan: m.makespan,
            mean_wait_time: m.mean_wait_time,
            gpu_utilization: m.gpu_utilization,
            jobs_completed: m.jobs_completed,
            jobs_total: m.jobs_total,
            mig_reconfigs: m.mig_reconfigs,
            preemptions: m.preemptions,
            topology_penalties: m.topology_penalties,
        }
    }
}

#[pymethods]
impl SimResult {
    fn __repr__(&self) -> String {
        format!(
            "SimResult(makespan={:.2}, mean_wait={:.2}, util={:.1}%, jobs={}/{})",
            self.makespan,
            self.mean_wait_time,
            self.gpu_utilization * 100.0,
            self.jobs_completed,
            self.jobs_total
        )
    }

    fn to_json(&self) -> String {
        let m = SimulationMetrics {
            makespan: self.makespan,
            mean_wait_time: self.mean_wait_time,
            gpu_utilization: self.gpu_utilization,
            jobs_completed: self.jobs_completed,
            jobs_total: self.jobs_total,
            queue_max_length: 0,
            mig_reconfigs: self.mig_reconfigs,
            preemptions: self.preemptions,
            topology_penalties: self.topology_penalties,
        };
        m.to_json_pretty()
    }
}

#[pyclass]
struct SimSession {
    inner: RlSession,
}

#[pymethods]
impl SimSession {
    #[new]
    fn new(config_path: &str) -> PyResult<Self> {
        let session = load_rl_session(std::path::Path::new(config_path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner: session })
    }

    #[getter]
    fn obs_size(&self) -> usize {
        self.inner.obs_size()
    }

    #[getter]
    fn action_space_n(&self) -> usize {
        self.inner.action_space_n()
    }

    #[getter]
    fn top_k(&self) -> usize {
        self.inner.top_k
    }

    #[getter]
    fn is_done(&self) -> bool {
        self.inner.is_done()
    }

    fn reset(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let snap = self.inner.reset();
        snapshot_to_py(py, &snap)
    }

    fn observe(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        snapshot_to_py(py, &self.inner.observe())
    }

    fn step(&mut self, py: Python<'_>, action: usize) -> PyResult<Py<PyAny>> {
        let result = self.inner.step(action);
        let dict = PyDict::new_bound(py);
        dict.set_item("observation", snapshot_to_py(py, &result.observation)?)?;
        dict.set_item("reward", result.reward)?;
        dict.set_item("done", result.done)?;
        dict.set_item("placed", result.placed)?;
        dict.set_item("invalid_action", result.invalid_action)?;
        Ok(dict.into())
    }

    fn metrics(&self) -> SimResult {
        SimResult::from(SimulationMetrics::from_cluster(
            &self.inner.cluster,
            self.inner.jobs_total,
        ))
    }
}

fn snapshot_to_py(py: Python<'_>, snap: &forgesim_core::ClusterSnapshot) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("clock", snap.clock)?;
    dict.set_item("free_gpus", snap.free_gpus)?;
    dict.set_item("waiting", snap.waiting)?;
    dict.set_item("running", snap.running)?;
    dict.set_item("finished", snap.finished)?;
    dict.set_item("features", snap.to_feature_vector())?;
    Ok(dict.into())
}

#[pyfunction]
fn run_from_config(config_path: &str) -> PyResult<SimResult> {
    let metrics = run_simulation(std::path::Path::new(config_path))
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(SimResult::from(metrics))
}

#[pymodule]
fn _forgesim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SimResult>()?;
    m.add_class::<SimSession>()?;
    m.add_function(wrap_pyfunction!(run_from_config, m)?)?;
    Ok(())
}
