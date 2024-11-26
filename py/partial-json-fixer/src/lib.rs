use original_partial_json_fixer;
use pyo3::prelude::*;

/// Fixes a partial json string to return a complete json string
#[pyfunction]
fn fix_json_string(partial_json: &str) -> PyResult<String> {
    Ok(original_partial_json_fixer::fix_json(partial_json))
}

/// A Python module implemented in Rust.
#[pymodule]
fn partial_json_fixer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fix_json_string, m)?)?;
    Ok(())
}
