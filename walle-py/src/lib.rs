use pyo3::prelude::*;

mod event;
pub mod extend;
pub mod message;

#[pymodule]
/// this is the main module of walle(a rusty libonebot)
fn walle(_py: Python, m: &PyModule) -> PyResult<()> {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    m.add_class::<event::Event>()?;
    Ok(())
}
