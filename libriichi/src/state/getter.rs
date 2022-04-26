use super::{ActionCandidate, PlayerState};
use crate::tile::Tile;

impl PlayerState {
    #[inline]
    pub const fn player_id(&self) -> u8 {
        self.player_id
    }
    #[inline]
    pub const fn is_oya(&self) -> bool {
        self.oya == 0
    }
    #[inline]
    pub const fn tehai(&self) -> [u8; 34] {
        self.arrs.tehai
    }
    #[inline]
    pub const fn akas_in_hand(&self) -> [bool; 3] {
        self.akas_in_hand
    }

    #[inline]
    pub fn chis(&self) -> &[u8] {
        &self.chis
    }
    #[inline]
    pub fn pons(&self) -> &[u8] {
        &self.pons
    }
    #[inline]
    pub fn minkans(&self) -> &[u8] {
        &self.minkans
    }
    #[inline]
    pub fn ankans(&self) -> &[u8] {
        &self.ankans
    }

    #[inline]
    pub const fn shanten(&self) -> i8 {
        self.shanten
    }
    #[inline]
    pub const fn waits(&self) -> [bool; 34] {
        self.arrs.waits
    }
    #[inline]
    pub const fn last_self_tsumo(&self) -> Option<Tile> {
        self.last_self_tsumo
    }
    #[inline]
    pub const fn last_kawa_tile(&self) -> Option<Tile> {
        self.last_kawa_tile
    }

    #[inline]
    pub const fn last_cans(&self) -> ActionCandidate {
        self.last_cans
    }
    #[inline]
    pub fn ankan_candidates(&self) -> &[u8] {
        &self.ankan_candidates
    }
    #[inline]
    pub fn kakan_candidates(&self) -> &[u8] {
        &self.kakan_candidates
    }

    #[inline]
    pub const fn can_w_riichi(&self) -> bool {
        self.can_w_riichi
    }
    #[inline]
    pub const fn self_riichi_declared(&self) -> bool {
        self.riichi_declared[0]
    }
    #[inline]
    pub const fn self_riichi_accepted(&self) -> bool {
        self.riichi_accepted[0]
    }

    #[inline]
    pub const fn at_furiten(&self) -> bool {
        self.at_furiten
    }
}
