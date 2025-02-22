use pyo3::prelude::*;

use std::ffi::CString;

pub(crate) fn add_submodule(
    py: Python<'_>,
    prefix: &str,
    super_mod: &Bound<'_, PyModule>,
    m: &Bound<'_, PyModule>,
) -> PyResult<()> {
    super_mod.add_submodule(m)?;

    let name = m.name()?;
    let script = CString::new(format!(
        "import sys; sys.modules['{prefix}.{name}'] = {name}; del sys"
    ))?;
    py.run(&script, None, Some(&super_mod.dict()))
}
