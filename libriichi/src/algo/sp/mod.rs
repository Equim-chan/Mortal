//! Rust port of nekobean's C++ implementation of his single-player mahjong
//! calculator. Some of the original comments are included.
//!
//! Source: <https://github.com/nekobean/mahjong-cpp>
//!
//! Major differences compared to the C++ version:
//! - Whenever shanten calculation is involved, all types of shanten will be
//!   considered (using `shanten::calc_all`). In the original version, you can
//!   only choose one of normal, chitoi and kokushi.
//! - When calculating uradora probs, the actual the number of tiles left is
//!   calculated and used, while the original version uses a fixed value of 121.
//! - Riichi is optional, so you can calculate values of a dama-preferred hand,
//!   although dama has no benefit at all in single-player mahjong.
//! - `max_tsumo` is set to the actual value, instead of the hardcoded 17 or 18
//!   in the original version. Not only does this reduce the amount of
//!   calculations, but more importantly, I think this is the theoretically
//!   correct way to calculate, since we keep track of the actual `tiles_seen`
//!   on board so we can have the accurate denominator when building the
//!   `tsumo_prob_table`.
//!
//! Other improvements:
//! - More aggressive compile-time optimizations.
//!
//! To reproduce the behavior of the original C++ version, set feature
//! `sp_reproduce_cpp_ver`.

mod calc;
mod candidate;
mod state;
mod tile;

pub use calc::SPCalculator;
pub use candidate::{Candidate, CandidateColumn};
pub use state::InitState;
pub use tile::RequiredTile;

#[cfg(feature = "sp_reproduce_cpp_ver")]
pub const MAX_TSUMOS_LEFT: usize = 18;
/// In practice, the max number of tsumos left should be 17, since the first
/// tsumo of oya is mandatory.
#[cfg(not(feature = "sp_reproduce_cpp_ver"))]
pub const MAX_TSUMOS_LEFT: usize = 17;

#[cfg(feature = "sp_reproduce_cpp_ver")]
const CALC_SHANTEN_FN: fn(&[u8; 34], u8) -> i8 = super::shanten::calc_normal;
#[cfg(not(feature = "sp_reproduce_cpp_ver"))]
const CALC_SHANTEN_FN: fn(&[u8; 34], u8) -> i8 = super::shanten::calc_all;
