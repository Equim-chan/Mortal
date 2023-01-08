#![deny(
    rust_2018_idioms,
    let_underscore_drop,
    clippy::must_use_candidate,
    clippy::redundant_else,
    clippy::manual_assert,
    clippy::manual_ok_or,
    clippy::needless_for_each,
    clippy::needless_continue,
    clippy::map_unwrap_or,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::inefficient_to_string,
    clippy::let_unit_value,
    clippy::cloned_instead_of_copied,
    clippy::debug_assert_with_mut_call,
    clippy::equatable_if_let,
    clippy::default_union_representation,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::lossy_float_literal,
    clippy::implicit_clone,
    clippy::implicit_saturating_sub,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::string_lit_as_bytes,
    clippy::trivially_copy_pass_by_ref,
    clippy::unicode_not_nfc,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::useless_let_if_seq,
    clippy::mut_mut,
    clippy::nonstandard_macro_braces,
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr
)]
#![allow(clippy::borrow_deref_ref)] // FIXME: pyo3 code makes it complains

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
    pyo3_log::init();

    let name = m.name()?;
    if cfg!(debug_assertions) {
        eprintln!("{name}: this is a debug build.");
        m.add("__profile__", "debug")?;
    } else {
        m.add("__profile__", "release")?;
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
