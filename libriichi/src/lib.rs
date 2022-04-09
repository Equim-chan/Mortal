mod agent;
mod arena;
mod consts;
mod dataset;
mod macros;
mod mjai_bot;
mod py_helper;
mod tile;
mod vec_ops;

// pub for bins
pub mod mjai;
pub mod stat;
pub mod state;

// pub for benchmarks
pub mod algo;
pub mod hand;

use env_logger::Env;
use pyo3::prelude::*;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// This module provides implementations of the riichi mahjong including the
/// following features:
///
/// - (primary usage) Player state maintenance driven by mjai events (via
///   `state.PlayerState`).
/// - Read mjai logs and produce a batch of instances for training (via
///   `dataset`).
/// - Self-play under standard Tenhou rules (via `arena`).
/// - Definitions of observation and action space (via `consts`)
/// - Statistical works on mjai log (via `Stat`)
#[pymodule]
fn libriichi(py: Python, m: &PyModule) -> PyResult<()> {
    let name = m.name()?;
    if cfg!(debug_assertions) {
        eprintln!("{name}: this is a debug build.");
        m.add("__profile__", "debug")?;
        env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp_millis()
            .init();
    } else {
        m.add("__profile__", "release")?;
        env_logger::Builder::from_env(Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
    }
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    crate::algo::shanten::ensure_init();
    crate::algo::agari::ensure_init();

    crate::consts::register_module(py, name, m)?;
    crate::state::register_module(py, name, m)?;
    crate::dataset::register_module(py, name, m)?;
    crate::arena::register_module(py, name, m)?;

    m.add_class::<crate::stat::Stat>()?;
    m.add_class::<crate::mjai_bot::Bot>()?;

    Ok(())
}
