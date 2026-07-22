use forgesim_config::run_simulation;
use forgesim_metrics::SimulationMetrics;
use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;

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
}

impl From<SimulationMetrics> for SimResult {
    fn from(m: SimulationMetrics) -> Self {
        Self {
            makespan: m.makespan,
            mean_wait_time: m.mean_wait_time,
            gpu_utilization: m.gpu_utilization,
            jobs_completed: m.jobs_completed,
            jobs_total: m.jobs_total,
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
        };
        m.to_json_pretty()
    }
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
    m.add_function(wrap_pyfunction!(run_from_config, m)?)?;
    Ok(())
}
