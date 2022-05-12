use pyo3::prelude::*;

pub(crate) fn add_submodule(
    py: Python<'_>,
    prefix: &str,
    super_mod: &PyModule,
    m: &PyModule,
) -> PyResult<()> {
    super_mod.add_submodule(m)?;

    let name = m.name()?;
    let script = format!("import sys; sys.modules['{prefix}.{name}'] = {name}; del sys");
    py.run(&script, None, Some(super_mod.dict()))
}
