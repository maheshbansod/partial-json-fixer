use std::collections::HashMap;

use original_partial_json_fixer::{self, JsonArray, JsonObject, JsonValue};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyList, PyNone},
};

/// Fixes a partial json string to return a complete json string
#[pyfunction]
fn fix_json_string(partial_json: &str) -> PyResult<String> {
    Ok(original_partial_json_fixer::fix_json(partial_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?
        .to_string())
}

/// Fixes a partial json string to return a complete json string
#[pyfunction]
fn fix_json_object(partial_json: &str) -> PyResult<RJsonValue> {
    let value = original_partial_json_fixer::fix_json(partial_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(RJsonValue(value))
}

struct RJsonValue<'a>(JsonValue<'a>);

impl<'a> IntoPy<PyObject> for RJsonValue<'a> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self.0 {
            JsonValue::Array(JsonArray { members }) => {
                PyList::new_bound(py, members.into_iter().map(|m| RJsonValue(m).into_py(py))).into()
            }
            JsonValue::Object(JsonObject { values }) => {
                let mut py_values = HashMap::new();
                for (key, value) in values.into_iter() {
                    let value = RJsonValue(value);
                    let key = unit_to_str(key);
                    py_values.insert(key, value);
                }
                py_values.into_py(py)
            }
            JsonValue::Unit(unit) => unit_to_py(unit, py),
            JsonValue::Null => PyNone::get_bound(py).into_py(py),
        }
    }
}
fn unit_to_str(unit: &str) -> &str {
    if unit.starts_with("\"") {
        let s = unit.trim_matches('"');
        s
    } else {
        unit
    }
}
fn unit_to_py(unit: &str, py: Python<'_>) -> PyObject {
    if unit.starts_with("\"") {
        let s = unit.trim_matches('"');
        s.into_py(py)
    } else if let Ok(unit) = unit.parse::<i32>() {
        unit.into_py(py)
    } else {
        PyNone::get_bound(py).into_py(py)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn partial_json_fixer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fix_json_string, m)?)?;
    m.add_function(wrap_pyfunction!(fix_json_object, m)?)?;
    Ok(())
}
