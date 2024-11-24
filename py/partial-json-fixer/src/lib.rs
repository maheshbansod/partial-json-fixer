use original_partial_json_fixer;
use pyo3::{exceptions::PyValueError, prelude::*};

/// Fixes a partial json string to return a complete json string
#[pyfunction]
fn fix_json_string(partial_json: &str) -> PyResult<String> {
    Ok(original_partial_json_fixer::fix_json(partial_json)
        .map_err(|err| PyValueError::new_err(err.to_string()))?.to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn partial_json_fixer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fix_json_string, m)?)?;
    Ok(())
}
