#![deny(
    rust_2018_idioms,
    clippy::must_use_candidate,
    clippy::redundant_else,
    clippy::manual_assert,
    clippy::needless_for_each,
    clippy::unnecessary_wraps,
    clippy::needless_pass_by_value,
    clippy::map_unwrap_or
)]

mod arena;
mod consts;
mod dataset;
mod macros;
mod py_helper;
mod vec_ops;

// pub for bins
pub mod chi_type;
pub mod mjai;
pub mod stat;
pub mod state;

// pub for non-cfg(test) tests
pub mod agent;
pub mod tile;

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
/// - The core feature - player state maintenance driven by mjai events (via
///   `state.PlayerState`).
/// - Read mjai logs and produce a batch of instances for training (via
///   `dataset`).
/// - Self-play under standard Tenhou rules (via `arena`).
/// - Definitions of observation and action space for Mortal (via `consts`).
/// - Statistical works on mjai logs (via `stat.Stat`).
/// - mjai interface (via `mjai.Bot`).
#[pymodule]
fn libriichi(py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

    algo::shanten::ensure_init();
    algo::agari::ensure_init();

    consts::register_module(py, name, m)?;
    state::register_module(py, name, m)?;
    dataset::register_module(py, name, m)?;
    arena::register_module(py, name, m)?;
    stat::register_module(py, name, m)?;
    mjai::register_module(py, name, m)?;

    Ok(())
}
