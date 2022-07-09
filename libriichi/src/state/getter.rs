use super::{ActionCandidate, PlayerState};
use crate::tile::Tile;

impl PlayerState {
    #[inline]
    #[must_use]
    pub const fn player_id(&self) -> u8 {
        self.player_id
    }
    #[inline]
    #[must_use]
    pub const fn is_oya(&self) -> bool {
        self.oya == 0
    }
    #[inline]
    #[must_use]
    pub const fn tehai(&self) -> [u8; 34] {
        self.tehai
    }
    #[inline]
    #[must_use]
    pub const fn akas_in_hand(&self) -> [bool; 3] {
        self.akas_in_hand
    }

    #[inline]
    #[must_use]
    pub fn chis(&self) -> &[u8] {
        &self.chis
    }
    #[inline]
    #[must_use]
    pub fn pons(&self) -> &[u8] {
        &self.pons
    }
    #[inline]
    #[must_use]
    pub fn minkans(&self) -> &[u8] {
        &self.minkans
    }
    #[inline]
    #[must_use]
    pub fn ankans(&self) -> &[u8] {
        &self.ankans
    }

    #[inline]
    #[must_use]
    pub const fn at_turn(&self) -> u8 {
        self.at_turn
    }
    #[inline]
    #[must_use]
    pub const fn shanten(&self) -> i8 {
        self.shanten
    }
    #[inline]
    #[must_use]
    pub const fn waits(&self) -> [bool; 34] {
        self.waits
    }
    #[inline]
    #[must_use]
    pub const fn last_self_tsumo(&self) -> Option<Tile> {
        self.last_self_tsumo
    }
    #[inline]
    #[must_use]
    pub const fn last_kawa_tile(&self) -> Option<Tile> {
        self.last_kawa_tile
    }

    #[inline]
    #[must_use]
    pub const fn last_cans(&self) -> ActionCandidate {
        self.last_cans
    }
    #[inline]
    #[must_use]
    pub fn ankan_candidates(&self) -> &[Tile] {
        &self.ankan_candidates
    }
    #[inline]
    #[must_use]
    pub fn kakan_candidates(&self) -> &[Tile] {
        &self.kakan_candidates
    }

    #[inline]
    #[must_use]
    pub const fn can_w_riichi(&self) -> bool {
        self.can_w_riichi
    }
    #[inline]
    #[must_use]
    pub const fn self_riichi_declared(&self) -> bool {
        self.riichi_declared[0]
    }
    #[inline]
    #[must_use]
    pub const fn self_riichi_accepted(&self) -> bool {
        self.riichi_accepted[0]
    }

    #[inline]
    #[must_use]
    pub const fn at_furiten(&self) -> bool {
        self.at_furiten
    }
}
