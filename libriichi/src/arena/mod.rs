mod board;
mod game;
mod one_vs_three;
mod result;
mod two_vs_two;

pub use board::Board;
pub use result::GameResult;

use crate::py_helper::add_submodule;
use one_vs_three::OneVsThree;
use two_vs_two::TwoVsTwo;

use pyo3::prelude::*;

pub(crate) fn register_module(
    py: Python<'_>,
    prefix: &str,
    super_mod: &Bound<'_, PyModule>,
) -> PyResult<()> {
    let m = PyModule::new(py, "arena")?;
    m.add_class::<OneVsThree>()?;
    m.add_class::<TwoVsTwo>()?;
    add_submodule(py, prefix, super_mod, &m)
}
