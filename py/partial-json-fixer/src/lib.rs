use std::collections::HashMap;

use original_partial_json_fixer::{self, JsonArray, JsonObject, JsonUnit, JsonValue};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBool, PyList, PyNone},
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
fn unit_to_str(unit: JsonUnit) -> &str {
    match unit {
        JsonUnit::Number(n) => n,
        JsonUnit::String(s) => s,
        _ => panic!("key of an object can only be string or number")
    }
}
fn unit_to_py(unit: JsonUnit<'_>, py: Python<'_>) -> PyObject {
    match unit {
        JsonUnit::Null => PyNone::get_bound(py).into_py(py),
        JsonUnit::True => PyBool::new_bound(py, true).into_py(py),
        JsonUnit::False => PyBool::new_bound(py, false).into_py(py),
        JsonUnit::String(s) => s.into_py(py),
        JsonUnit::Number(n) => n.into_py(py)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn partial_json_fixer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fix_json_string, m)?)?;
    m.add_function(wrap_pyfunction!(fix_json_object, m)?)?;
    Ok(())
}
