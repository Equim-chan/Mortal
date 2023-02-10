use crate::tile::Tile;
use std::fmt;

use serde::Serialize;
use tinyvec::ArrayVec;

#[derive(Debug, Clone, Serialize)]
pub(super) struct KawaItem {
    pub(super) chi_pon: Option<ChiPon>,
    pub(super) kan: ArrayVec<[Tile; 4]>,
    pub(super) sutehai: Sutehai,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct Sutehai {
    pub(super) tile: Tile,
    // only for normal dora, aka is not included
    pub(super) is_dora: bool,
    pub(super) is_tedashi: bool,
    pub(super) is_riichi: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ChiPon {
    pub(super) consumed: [Tile; 2],
    pub(super) target_tile: Tile,
}

impl fmt::Display for Sutehai {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.tile,
            if self.is_dora { "!" } else { "" },
            if self.is_tedashi { "" } else { "^" },
            if self.is_riichi { "|" } else { "" },
        )
    }
}

impl fmt::Display for ChiPon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}{}+{})",
            self.consumed[0], self.consumed[1], self.target_tile,
        )
    }
}

impl fmt::Display for KawaItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.kan.is_empty() {
            f.write_str("{")?;
            for kan in self.kan {
                write!(f, "{kan}")?;
            }
            f.write_str("}")?;
        }

        if let Some(chi_pon) = &self.chi_pon {
            write!(f, "{chi_pon}")?;
        }

        write!(f, "{}", self.sutehai)
    }
}
