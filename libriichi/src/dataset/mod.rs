//! Sample extractions.

mod gameplay;
mod grp;
mod player_list;

use crate::py_helper::add_submodule;
pub use gameplay::{Gameplay, GameplayLoader, Invisible};
pub use grp::Grp;

use pyo3::prelude::*;

pub(crate) fn register_module(py: Python, prefix: &str, super_mod: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "dataset")?;
    m.add_class::<Gameplay>()?;
    m.add_class::<GameplayLoader>()?;
    m.add_class::<Grp>()?;
    add_submodule(py, prefix, super_mod, m)
}
