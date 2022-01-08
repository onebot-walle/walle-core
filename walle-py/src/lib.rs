use pyo3::prelude::*;
use std::sync::Arc;

#[pymodule]
/// this is the main module of walle(a rusty libonebot)
fn walle(_py: Python, m: &PyModule) -> PyResult<()> {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    #[pyfn(m)]
    /// just build and run a onebot application
    fn run_onebot_app(py: Python) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async {
            let app = walle_core::app::OneBot::new(
                walle_core::AppConfig::default(),
                walle_core::DefaultHandler::arc(),
            )
            .arc();
            app.run().await.unwrap();
            Ok(PyOneBot(OBApp::V12(app)))
        })
    }
    #[pyfn(m)]
    /// just build and run a onebot11 application
    fn run_onebot11_app(py: Python) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async {
            let app = walle_v11::app::OneBot11::new(
                walle_core::AppConfig::default(),
                walle_v11::DefaultHandler::arc(),
            )
            .arc();
            app.run().await.unwrap();
            Ok(PyOneBot(OBApp::V11(app)))
        })
    }
    #[pyfn(m)]
    /// just build and run a onebot11 application
    fn run_block_onebot11_app(py: Python) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async {
            let app = walle_v11::app::OneBot11::new(
                walle_core::AppConfig::default(),
                walle_v11::DefaultHandler::arc(),
            )
            .arc();
            app.run_block().await.unwrap();
            Ok(())
        })
    }
    Ok(())
}

enum OBApp {
    V11(Arc<walle_v11::app::OneBot11>),
    V12(Arc<walle_core::app::OneBot>),
}

#[pyclass(name = "OneBot")]
struct PyOneBot(OBApp);

#[pymethods]
impl PyOneBot {
    // #[new]
    // fn __new__(_wraps: Py<PyAny>) -> Self {
    //     Self {
    //         inner: walle_core::app::OneBot::new(
    //             walle_core::AppConfig::default(),
    //             walle_core::DefaultHandler::arc(),
    //         )
    //         .arc(),
    //     }
    // }

    // fn __call__(&mut self, py: Python) -> PyResult<Py<PyAny>> {
    //     walle_core::app::OneBot::run(self.inner.clone());
    // }
}
