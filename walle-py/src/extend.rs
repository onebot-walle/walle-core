use std::collections::HashMap;

use pyo3::prelude::*;
use walle_core::ExtendedValue;

#[derive(Clone)]
pub struct PyExtendedMap(pub ExtendedValue);

impl IntoPy<PyObject> for PyExtendedMap {
    fn into_py(self, py: Python) -> PyObject {
        match self.0 {
            ExtendedValue::Int(i) => i.into_py(py),
            ExtendedValue::F64(f) => f.into_py(py),
            ExtendedValue::Str(s) => s.into_py(py),
            ExtendedValue::Bool(b) => b.into_py(py),
            ExtendedValue::List(l) => l
                .into_iter()
                .map(|v| Self(v))
                .collect::<Vec<_>>()
                .into_py(py),
            ExtendedValue::Map(m) => m
                .into_iter()
                .fold(HashMap::new(), |mut map, (k, v)| {
                    map.insert(k, Self(v));
                    map
                })
                .into_py(py),
            ExtendedValue::Null => None::<Option<Self>>.into_py(py),
        }
    }
}

impl<'s> FromPyObject<'s> for PyExtendedMap {
    fn extract(s: &'s PyAny) -> PyResult<Self> {
        if let Ok(i) = s.extract() {
            return Ok(Self(ExtendedValue::Int(i)));
        } else if let Ok(f) = s.extract() {
            return Ok(Self(ExtendedValue::F64(f)));
        } else if let Ok(s) = s.extract() {
            return Ok(Self(ExtendedValue::Str(s)));
        } else if let Ok(b) = s.extract() {
            return Ok(Self(ExtendedValue::Bool(b)));
        } else if let Ok(l) = s.extract::<'_, Vec<Self>>() {
            return Ok(Self(ExtendedValue::List(
                l.into_iter().map(|v| v.0).collect(),
            )));
        } else if let Ok(m) = s.extract::<'_, HashMap<String, Self>>() {
            return Ok(Self(ExtendedValue::Map(m.into_iter().fold(
                HashMap::default(),
                |mut map, (k, v)| {
                    map.insert(k, v.0);
                    map
                },
            ))));
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "invalid type",
            ));
        }
    }
}
