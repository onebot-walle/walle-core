use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn run_onebot_app(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        let app = walle_core::app::OneBot::new(
            walle_core::AppConfig::default(),
            walle_core::DefaultHandler::arc(),
        )
        .arc();
        walle_core::app::OneBot::run(app).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn walle(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_onebot_app, m)?)?;
    Ok(())
}
