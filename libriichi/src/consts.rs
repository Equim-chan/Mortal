use crate::py_helper::add_submodule;

use pyo3::prelude::*;
use static_assertions::const_assert;

pub const OBS_SHAPE: (usize, usize) = (938, 34);
pub const ORACLE_OBS_SHAPE: (usize, usize) = (211, 34);
pub const ACTION_SPACE: usize = 37 // discard | kan (choice)
                              + 1  // riichi
                              + 3  // chi
                              + 1  // pon
                              + 1  // kan (decide)
                              + 1  // agari
                              + 1  // ryukyoku
                              + 1; // pass
                                   // = 46
pub const GRP_SIZE: usize = 7;

const_assert!(ACTION_SPACE <= u64::BITS as usize);

pub(crate) fn register_module(py: Python<'_>, prefix: &str, super_mod: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "consts")?;
    m.add("OBS_SHAPE", OBS_SHAPE)?;
    m.add("ORACLE_OBS_SHAPE", ORACLE_OBS_SHAPE)?;
    m.add("ACTION_SPACE", ACTION_SPACE)?;
    m.add("GRP_SIZE", GRP_SIZE)?;
    add_submodule(py, prefix, super_mod, m)
}
