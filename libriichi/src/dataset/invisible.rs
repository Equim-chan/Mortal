use crate::arena::Board;
use crate::consts::oracle_obs_shape;
use crate::mjai::Event;
use crate::state::PlayerState;
use crate::tile::Tile;
use crate::{must_tile, tu8, tuz};
use std::iter;
use std::mem;

use ndarray::prelude::*;
use rand::prelude::*;

/// All fields are sorted early -> late.
#[derive(Default)]
pub struct Invisible {
    pub yama: Vec<Tile>,
    pub rinshan: Vec<Tile>,
    pub dora_indicators: Vec<Tile>,
    pub ura_indicators: Vec<Tile>,
}

impl Invisible {
    pub fn new(game: &[Event], trust_seed: bool) -> Vec<Self> {
        let mut ret = vec![];
        let mut cur = Self::default();
        let mut seed = None;
        let mut from_rinshan = false;
        let mut ura_is_recorded = false;
        let mut unknown_tiles = new_unknown_tiles();

        for event in game {
            match event {
                // If the game was emulated by our lib, then use the seed directly
                Event::StartGame {
                    seed: Some(game_seed),
                    ..
                } if trust_seed => {
                    seed = Some(*game_seed);
                }

                Event::StartKyoku {
                    bakaze,
                    kyoku,
                    honba,
                    dora_marker,
                    tehais,
                    ..
                } => {
                    if let Some(seed) = seed {
                        let mut board = Board {
                            kyoku: 4 * (bakaze.as_u8() - tu8!(E)) + kyoku - 1,
                            honba: *honba,
                            ..Default::default()
                        };
                        board.init_from_seed(seed);

                        cur.yama = board.yama;
                        cur.rinshan = board.rinshan;
                        cur.dora_indicators = board.dora_indicators;
                        cur.ura_indicators = board.ura_indicators;

                        // reverse because of the way Board pops tiles
                        cur.yama.reverse();
                        cur.rinshan.reverse();
                        cur.dora_indicators.reverse();

                        ret.push(mem::take(&mut cur));
                        continue;
                    }
                    cur.dora_indicators.push(*dora_marker);
                    unknown_tiles[dora_marker.as_usize()] -= 1;
                    tehais
                        .iter()
                        .flatten()
                        .for_each(|tile| unknown_tiles[tile.as_usize()] -= 1);
                }
                _ => (),
            };

            if seed.is_some() {
                continue;
            }

            match event {
                Event::Tsumo { pai, .. } => {
                    if from_rinshan {
                        cur.rinshan.push(*pai);
                        from_rinshan = false;
                    } else {
                        cur.yama.push(*pai);
                        assert!(cur.yama.len() <= 70, "yama size overflow");
                    }
                    unknown_tiles[pai.as_usize()] -= 1;
                }
                Event::Ankan { .. } | Event::Kakan { .. } | Event::Daiminkan { .. } => {
                    from_rinshan = true;
                }
                Event::Dora { dora_marker } => {
                    cur.dora_indicators.push(*dora_marker);
                    unknown_tiles[dora_marker.as_usize()] -= 1;
                }
                Event::Hora {
                    ura_markers: Some(ura),
                    ..
                } if !ura_is_recorded => {
                    for &tile in ura {
                        cur.ura_indicators.push(tile);
                        unknown_tiles[tile.as_usize()] -= 1;
                    }
                    ura_is_recorded = true;
                }
                Event::EndKyoku => {
                    let mut filler: Vec<_> = unknown_tiles
                        .into_iter()
                        .enumerate()
                        .filter(|&(_, count)| count > 0)
                        .flat_map(|(tid, count)| iter::repeat(must_tile!(tid)).take(count as usize))
                        .collect();
                    filler.shuffle(&mut thread_rng());

                    while cur.yama.len() < 70 {
                        cur.yama.push(filler.pop().unwrap());
                    }
                    while cur.rinshan.len() < 4 {
                        cur.rinshan.push(filler.pop().unwrap());
                    }
                    while cur.dora_indicators.len() < 5 {
                        cur.dora_indicators.push(filler.pop().unwrap());
                    }
                    while cur.ura_indicators.len() < 5 {
                        cur.ura_indicators.push(filler.pop().unwrap());
                    }
                    assert!(filler.is_empty());

                    ret.push(mem::take(&mut cur));
                    from_rinshan = false;
                    ura_is_recorded = false;
                    unknown_tiles = new_unknown_tiles();
                }

                _ => (),
            };
        }

        ret
    }

    // TODO: merge this this arena::board::BoardState::encode_oracle_obs; they
    // should be identical.
    pub fn encode(
        &self,
        opponent_states: &[PlayerState; 3],
        yama_idx: usize,
        rinshan_idx: usize,
        version: u32,
    ) -> Array2<f32> {
        let shape = oracle_obs_shape(version);
        let mut arr = Array2::zeros(shape);
        let mut idx = 0;

        for state in opponent_states {
            state
                .tehai()
                .iter()
                .enumerate()
                .filter(|(_, &count)| count > 0)
                .for_each(|(tile_id, &count)| {
                    arr.slice_mut(s![idx..idx + count as usize, tile_id])
                        .fill(1.);
                });
            idx += 4;

            state
                .akas_in_hand()
                .iter()
                .enumerate()
                .filter(|(_, &has_it)| has_it)
                .for_each(|(i, _)| {
                    arr.slice_mut(s![idx + i, ..]).fill(1.);
                });
            idx += 3;

            let n = state.shanten() as usize;
            match version {
                1 => {
                    arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
                    idx += 6;
                }
                2 | 3 => {
                    arr.slice_mut(s![idx + n, ..]).fill(1.);
                    idx += 7;

                    let v = n as f32 / 6.;
                    arr.slice_mut(s![idx, ..]).fill(v);
                    idx += 1;
                }
                _ => unreachable!(),
            }

            state
                .waits()
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| arr[[idx, t]] = 1.);
            idx += 1;

            if state.at_furiten() {
                arr.slice_mut(s![idx, ..]).fill(1.);
            }
            idx += 1;
        }

        let mut encode_tile = |idx: usize, tile: Tile| {
            let tile_id = tile.deaka().as_usize();
            arr[[idx, tile_id]] = 1.;
            if tile.is_aka() {
                arr.slice_mut(s![idx + 1, ..]).fill(1.);
            }
        };

        for &tile in &self.yama[yama_idx..] {
            encode_tile(idx, tile);
            idx += 2;
        }
        // In real life case `self.yama[yama_idx..]` is at most 69 (`yama_idx`
        // is always >= 1), because the dealer always unconditionally deals the
        // first tile from yama. Therefore we do the minus one here.
        idx += (yama_idx - 1) * 2;

        for &tile in &self.rinshan[rinshan_idx..] {
            encode_tile(idx, tile);
            idx += 2;
        }
        idx += rinshan_idx * 2;

        for &tile in &self.dora_indicators {
            encode_tile(idx, tile);
            idx += 2;
        }
        for &tile in &self.ura_indicators {
            encode_tile(idx, tile);
            idx += 2;
        }

        assert_eq!(idx, shape.0);
        arr
    }
}

const fn new_unknown_tiles() -> [u8; 37] {
    let mut ret = [4; 37];
    ret[tuz!(5m)] = 3;
    ret[tuz!(5p)] = 3;
    ret[tuz!(5s)] = 3;
    ret[tuz!(5mr)] = 1;
    ret[tuz!(5pr)] = 1;
    ret[tuz!(5sr)] = 1;
    ret
}
