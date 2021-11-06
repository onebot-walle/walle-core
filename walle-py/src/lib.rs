// use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;

// #[pyfunction]
// /// just build and run a onebot application
// fn run_onebot_app(py: Python) -> PyResult<&PyAny> {
//     pyo3_asyncio::tokio::future_into_py(py, async {
//         let app = walle_core::app::OneBot::new(
//             walle_core::AppConfig::default(),
//             walle_core::DefaultHandler::arc(),
//         )
//         .arc();
//         walle_core::app::OneBot::run(app).await.unwrap();
//         Ok(Python::with_gil(|py| py.None()))
//     })
// }

// #[pymodule]
// /// this is the main module of walle(a rusty libonebot)
// fn walle(_py: Python, m: &PyModule) -> PyResult<()> {
//     let env = tracing_subscriber::EnvFilter::from("walle_core=trace,Walle-core=trace");
//     tracing_subscriber::fmt().with_env_filter(env).init();
//     m.add_function(wrap_pyfunction!(run_onebot_app, m)?)?;
//     Ok(())
// }

// #[pyclass(name = "OneBot")]
// struct PyOneBot {
//     inner: Arc<walle_core::app::OneBot>,
// }

// #[pymethods]
// impl PyOneBot {
//     #[new]
//     fn __new__(_wraps: Py<PyAny>) -> Self {
//         Self {
//             inner: walle_core::app::OneBot::new(
//                 walle_core::AppConfig::default(),
//                 walle_core::DefaultHandler::arc(),
//             )
//             .arc(),
//         }
//     }

//     fn __call__(&mut self, py: Python) -> PyResult<Py<PyAny>> {
//         walle_core::app::OneBot::run(self.inner.clone());
//     }
// }
