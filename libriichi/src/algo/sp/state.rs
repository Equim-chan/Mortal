use super::CALC_SHANTEN_FN;
use super::tile::{DiscardTile, DrawTile, RequiredTile};
use crate::tile::Tile;
use crate::{must_tile, t, tu8};

use tinyvec::ArrayVec;

/// Mutable state of both the hand and the board.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct State {
    // hand
    pub(super) tehai: [u8; 34],
    pub(super) akas_in_hand: [bool; 3],

    // global
    pub(super) tiles_in_wall: [u8; 34],
    pub(super) akas_in_wall: [bool; 3],
    pub(super) n_extra_tsumo: u8,
}

/// Mutable state of both the hand and the board.
#[derive(Clone)]
pub struct InitState {
    // hand
    pub tehai: [u8; 34],
    pub akas_in_hand: [bool; 3],

    // global
    pub tiles_seen: [u8; 34],
    pub akas_seen: [bool; 3],
}

impl From<InitState> for State {
    fn from(
        InitState {
            tehai,
            akas_in_hand,
            tiles_seen,
            akas_seen,
        }: InitState,
    ) -> Self {
        let mut tiles_in_wall = tiles_seen;
        let mut akas_in_wall = akas_seen;
        tiles_in_wall.iter_mut().for_each(|v| *v = 4 - *v);
        akas_in_wall.iter_mut().for_each(|v| *v = !*v);
        Self {
            tehai,
            akas_in_hand,
            tiles_in_wall,
            akas_in_wall,
            n_extra_tsumo: 0,
        }
    }
}

impl State {
    pub(super) const fn discard(&mut self, tile: Tile) {
        self.tehai[tile.deaka().as_usize()] -= 1;
        match tile.as_u8() {
            tu8!(5mr) => self.akas_in_hand[0] = false,
            tu8!(5pr) => self.akas_in_hand[1] = false,
            tu8!(5sr) => self.akas_in_hand[2] = false,
            _ => (),
        }
    }

    pub(super) const fn undo_discard(&mut self, tile: Tile) {
        self.tehai[tile.deaka().as_usize()] += 1;
        match tile.as_u8() {
            tu8!(5mr) => self.akas_in_hand[0] = true,
            tu8!(5pr) => self.akas_in_hand[1] = true,
            tu8!(5sr) => self.akas_in_hand[2] = true,
            _ => (),
        }
    }

    pub(super) const fn deal(&mut self, tile: Tile) {
        self.tiles_in_wall[tile.deaka().as_usize()] -= 1;
        match tile.as_u8() {
            tu8!(5mr) => self.akas_in_wall[0] = false,
            tu8!(5pr) => self.akas_in_wall[1] = false,
            tu8!(5sr) => self.akas_in_wall[2] = false,
            _ => (),
        }
        self.undo_discard(tile);
    }

    pub(super) const fn undo_deal(&mut self, tile: Tile) {
        self.discard(tile);
        self.tiles_in_wall[tile.deaka().as_usize()] += 1;
        match tile.as_u8() {
            tu8!(5mr) => self.akas_in_wall[0] = true,
            tu8!(5pr) => self.akas_in_wall[1] = true,
            tu8!(5sr) => self.akas_in_wall[2] = true,
            _ => (),
        }
    }

    pub(super) fn get_discard_tiles(
        &self,
        shanten: i8,
        tehai_len_div3: u8,
    ) -> ArrayVec<[DiscardTile; 14]> {
        let mut discard_tiles = ArrayVec::default();

        let mut tehai = self.tehai;
        for tid in 0..34 {
            if tehai[tid] == 0 {
                continue;
            }

            tehai[tid] -= 1;
            let shanten_after = CALC_SHANTEN_FN(&tehai, tehai_len_div3);
            tehai[tid] += 1;

            let shanten_diff = shanten_after - shanten;

            let tile = match tid as u8 {
                tu8!(5m) if self.akas_in_hand[0] && tehai[tid] == 1 => t!(5mr),
                tu8!(5p) if self.akas_in_hand[1] && tehai[tid] == 1 => t!(5pr),
                tu8!(5s) if self.akas_in_hand[2] && tehai[tid] == 1 => t!(5sr),
                _ => must_tile!(tid),
            };

            discard_tiles.push(DiscardTile { tile, shanten_diff });
        }

        discard_tiles
    }

    pub(super) fn get_draw_tiles(
        &self,
        shanten: i8,
        tehai_len_div3: u8,
    ) -> ArrayVec<[DrawTile; 37]> {
        let mut draw_tiles = ArrayVec::default();

        let mut tehai = self.tehai;
        for (tid, &count) in self.tiles_in_wall.iter().enumerate() {
            if count == 0 {
                continue;
            }

            tehai[tid] += 1;
            let shanten_after = CALC_SHANTEN_FN(&tehai, tehai_len_div3);
            tehai[tid] -= 1;

            let shanten_diff = shanten_after - shanten;

            let tile = must_tile!(tid);
            match (tid as u8, self.akas_in_wall) {
                (tu8!(5m), [true, _, _]) | (tu8!(5p), [_, true, _]) | (tu8!(5s), [_, _, true]) => {
                    if count >= 2 {
                        draw_tiles.push(DrawTile {
                            tile,
                            count: count - 1,
                            shanten_diff,
                        });
                    }
                    draw_tiles.push(DrawTile {
                        tile: tile.akaize(),
                        count: 1,
                        shanten_diff,
                    });
                }
                _ => draw_tiles.push(DrawTile {
                    tile,
                    count,
                    shanten_diff,
                }),
            }
        }

        draw_tiles
    }

    pub(super) fn get_required_tiles(&self, tehai_len_div3: u8) -> ArrayVec<[RequiredTile; 34]> {
        let mut tehai = self.tehai;

        let shanten = CALC_SHANTEN_FN(&tehai, tehai_len_div3);
        let mut required_tiles = ArrayVec::default();

        for (tid, &count) in self.tiles_in_wall.iter().enumerate() {
            if count == 0 {
                continue;
            }

            tehai[tid] += 1;
            let shanten_after = CALC_SHANTEN_FN(&tehai, tehai_len_div3);
            tehai[tid] -= 1;

            if shanten_after < shanten {
                required_tiles.push(RequiredTile {
                    tile: must_tile!(tid),
                    count,
                });
            }
        }

        required_tiles
    }

    pub(super) fn sum_left_tiles(&self) -> u8 {
        self.tiles_in_wall.iter().sum()
    }
}
