use crate::extend::PyExtendedMap;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct MessageSegment {
    #[pyo3(get, set)]
    pub r#type: String,
    #[pyo3(get, set)]
    pub data: PyExtendedMap,
}
