use super::PlayerState;
use crate::consts::{ACTION_SPACE, OBS_SHAPE};
use crate::state::item::KawaItem;
use crate::{tu8, tuz};

use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;

#[pymethods]
impl PlayerState {
    /// Returns `(obs, mask)`
    #[pyo3(name = "encode_obs")]
    #[pyo3(text_signature = "($self, at_kan_select)")]
    fn encode_obs_py<'py>(
        &self,
        at_kan_select: bool,
        py: Python<'py>,
    ) -> (&'py PyArray2<f32>, &'py PyArray1<bool>) {
        let (obs, mask) = self.encode_obs(at_kan_select);
        let obs = PyArray2::from_owned_array(py, obs);
        let mask = PyArray1::from_owned_array(py, mask);
        (obs, mask)
    }
}

impl PlayerState {
    /// Returns `(obs, mask)`
    #[must_use]
    pub fn encode_obs(&self, at_kan_select: bool) -> (Array2<f32>, Array1<bool>) {
        let mut arr = Array2::zeros(OBS_SHAPE);
        let mut mask = Array1::default(ACTION_SPACE);
        let mut idx = 0;
        let cans = self.last_cans;

        self.tehai
            .iter()
            .enumerate()
            .filter(|(_, &count)| count > 0)
            .for_each(|(tile_id, &count)| {
                let n = count as usize;
                arr.slice_mut(s![idx..idx + n, tile_id]).fill(1.);
            });
        idx += 4;

        self.akas_in_hand
            .into_iter()
            .enumerate()
            .filter(|&(_, has_it)| has_it)
            .for_each(|(i, _)| {
                arr.slice_mut(s![idx + i, ..]).fill(1.);
            });
        idx += 3;

        for (i, &score) in self.scores.iter().enumerate() {
            let v = score.clamp(0, 100_000) as f32 / 100_000.;
            arr.slice_mut(s![idx + i, ..]).fill(v);
        }
        idx += 4;

        let i = self.rank as usize;
        arr.slice_mut(s![idx + i, ..]).fill(1.);
        idx += 4;

        let n = self.kyoku as usize;
        arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
        idx += 4;

        let n = self.honba.min(10) as usize;
        arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
        idx += 10;

        let n = self.kyotaku.min(10) as usize;
        arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
        idx += 10;

        arr[[idx, self.bakaze.as_usize()]] = 1.;
        arr[[idx + 1, self.jikaze.as_usize()]] = 1.;
        idx += 2;

        for tile in self.dora_indicators {
            let tile_id = tile.deaka().as_usize();
            let i = (0..4).find(|&i| arr[[idx + i, tile_id]] == 0.).unwrap();
            arr[[idx + i, tile_id]] = 1.;
            if tile.is_aka() {
                let i = tile.as_usize() - 34;
                arr.slice_mut(s![idx + 4 + i, ..]).fill(1.);
            }
        }
        idx += 7;

        let mut encode_my_kawa = |idx: usize, k: &KawaItem| {
            for kan in k.kan {
                // deaka is required, it is possible for it to be an aka
                // (for example in Daiminkan and Kakan).
                let tile_id = kan.deaka().as_usize();
                arr[[idx, tile_id]] = 1.;
            }

            let sutehai = &k.sutehai;
            let tile_id = sutehai.tile.deaka().as_usize();
            arr[[idx + 1, tile_id]] = 1.;
            if sutehai.tile.is_aka() {
                arr.slice_mut(s![idx + 2, ..]).fill(1.);
            }
            if sutehai.is_dora {
                arr.slice_mut(s![idx + 3, ..]).fill(1.);
            }
        };

        self.kawa[0].iter().take(6).for_each(|kawa_item| {
            if let Some(k) = &kawa_item {
                encode_my_kawa(idx, k);
            }
            idx += 4;
        });
        idx += (6 - self.kawa[0].len().min(6)) * 4;

        self.kawa[0].iter().rev().take(18).for_each(|kawa_item| {
            if let Some(k) = &kawa_item {
                encode_my_kawa(idx, k);
            }
            idx += 4;
        });
        idx += (18 - self.kawa[0].len().min(18)) * 4;

        let mut encode_kawa = |idx: usize, k: &KawaItem| {
            if let Some(cp) = &k.chi_pon {
                // Aka info of the chi/pon is not encoded in the kawa detail;
                // they are included in fuuro_overview instead.
                //
                // This is one-hot.
                let a = cp.consumed[0].deaka().as_usize();
                let b = cp.consumed[1].deaka().as_usize();
                let min = a.min(b);
                let max = a.max(b);
                arr[[idx, min]] = 1.;
                arr[[idx + 1, max]] = 1.;
            }

            for kan in k.kan {
                let tile_id = kan.deaka().as_usize();
                arr[[idx + 2, tile_id]] = 1.;
            }

            let tile_id = k.sutehai.tile.deaka().as_usize();
            arr[[idx + 3, tile_id]] = 1.;
            if k.sutehai.tile.is_aka() {
                arr.slice_mut(s![idx + 4, ..]).fill(1.);
            }
            if k.sutehai.is_dora {
                arr.slice_mut(s![idx + 5, ..]).fill(1.);
            }
            if k.sutehai.is_tedashi {
                arr.slice_mut(s![idx + 6, ..]).fill(1.);
            }
            if k.sutehai.is_riichi {
                arr.slice_mut(s![idx + 7, ..]).fill(1.);
            }
        };

        for player_kawa in &self.kawa[1..] {
            for kawa_item in player_kawa.iter().take(6) {
                if let Some(k) = kawa_item {
                    encode_kawa(idx, k);
                }
                idx += 8;
            }
            idx += (6 - player_kawa.len().min(6)) * 8;

            for kawa_item in player_kawa.iter().rev().take(18) {
                if let Some(k) = kawa_item {
                    encode_kawa(idx, k);
                }
                idx += 8;
            }
            idx += (18 - player_kawa.len().min(18)) * 8;
        }

        let v = self.tiles_left as f32 / 69.;
        arr.slice_mut(s![idx, ..]).fill(v);
        idx += 1;

        for count in self.doras_owned {
            let n = count.min(12) as usize;
            arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
            idx += 12;
        }

        let doras_unseen = self.dora_indicators.len() as u8 * 4 + 3 - self.doras_seen;
        let n = doras_unseen.min(5 * 4 + 3) as usize;
        arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
        idx += 5 * 4 + 3;

        for player_kawa_overview in &self.kawa_overview {
            for tile in player_kawa_overview {
                let tile_id = tile.deaka().as_usize();
                let i = (0..4).find(|&i| arr[[idx + i, tile_id]] == 0.).unwrap();
                arr[[idx + i, tile_id]] = 1.;
                if tile.is_aka() {
                    let i = tile.as_usize() - 34;
                    arr.slice_mut(s![idx + 4 + i, ..]).fill(1.);
                }
            }
            idx += 7;
        }

        for player_fuuro in &self.fuuro_overview {
            for f in player_fuuro {
                for tile in f {
                    let tile_id = tile.deaka().as_usize();
                    let i = (0..4).find(|&i| arr[[idx + i, tile_id]] == 0.).unwrap();
                    arr[[idx + i, tile_id]] = 1.;
                    // It is not possible to have more than one aka in a fuuro
                    // set, at least in tenhou rule, so we simply use one
                    // channel here.
                    if tile.is_aka() {
                        arr.slice_mut(s![idx + 4, ..]).fill(1.);
                    }
                }
                idx += 5;
            }
            idx += (4 - player_fuuro.len()) * 5;
        }

        for player_ankan in &self.ankan_overview {
            for tile in player_ankan {
                let tile_id = tile.as_usize();
                arr[[idx, tile_id]] = 1.;
            }
            idx += 1;
        }

        self.riichi_declared[1..]
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .for_each(|(i, _)| arr.slice_mut(s![idx + i, ..]).fill(1.));
        idx += 3;
        self.riichi_accepted[1..]
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .for_each(|(i, _)| arr.slice_mut(s![idx + i, ..]).fill(1.));
        idx += 3;

        self.waits
            .iter()
            .enumerate()
            .filter(|(_, &c)| c)
            .for_each(|(t, _)| arr[[idx, t]] = 1.);
        idx += 1;

        if self.at_furiten {
            arr.slice_mut(s![idx, ..]).fill(1.);
        }
        idx += 1;

        let n = self.shanten as usize;
        arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
        idx += 6;

        if self.riichi_accepted[0] {
            arr.slice_mut(s![idx, ..]).fill(1.);
        }
        idx += 1;

        if at_kan_select {
            arr.slice_mut(s![idx, ..]).fill(1.);
        }
        idx += 1;

        if cans.can_chi() || cans.can_pon || cans.can_daiminkan || cans.can_ron_agari {
            let tile = self
                .last_kawa_tile
                .expect("building chi/pon/daiminkan/ron feature without any kawa tile");
            let tile_id = tile.deaka().as_usize();

            arr[[idx, tile_id]] = 1.;
            if tile.is_aka() {
                arr.slice_mut(s![idx + 1, ..]).fill(1.);
            }
            if self.dora_factor[tile.deaka().as_usize()] > 0 {
                arr.slice_mut(s![idx + 2, ..]).fill(1.);
            }

            // pass
            if !at_kan_select {
                mask[ACTION_SPACE - 1] = true;
            } else if cans.can_daiminkan {
                mask[tile_id] = true;
            }
        }
        idx += 3;

        if cans.can_discard {
            self.discard_candidates_aka()
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| {
                    let deaka_t = match t as u8 {
                        tu8!(5mr) => tuz!(5m),
                        tu8!(5pr) => tuz!(5p),
                        tu8!(5sr) => tuz!(5s),
                        _ => t,
                    };
                    arr[[idx, deaka_t]] = 1.;
                    if !at_kan_select {
                        mask[t] = true;
                    }
                });

            self.keep_shanten_discards
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| arr[[idx + 1, t]] = 1.);
            self.next_shanten_discards
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| arr[[idx + 2, t]] = 1.);

            if self.shanten <= 1 {
                self.discard_candidates_with_unconditional_tenpai()
                    .iter()
                    .enumerate()
                    .filter(|(_, &c)| c)
                    .for_each(|(t, _)| arr[[idx + 3, t]] = 1.);
            }

            if self.riichi_declared[0] {
                arr.slice_mut(s![idx + 4, ..]).fill(1.);
            }
        }
        idx += 5;

        if cans.can_riichi {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[37] = true;
            }
        }
        idx += 1;

        if cans.can_chi_low {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[38] = true;
            }
        }
        if cans.can_chi_mid {
            arr.slice_mut(s![idx + 1, ..]).fill(1.);
            if !at_kan_select {
                mask[39] = true;
            }
        }
        if cans.can_chi_high {
            arr.slice_mut(s![idx + 2, ..]).fill(1.);
            if !at_kan_select {
                mask[40] = true;
            }
        }
        idx += 3;

        if cans.can_pon {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[41] = true;
            }
        }
        idx += 1;

        if cans.can_daiminkan {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[42] = true;
            }
        }
        idx += 1;

        if cans.can_ankan {
            for tile in self.ankan_candidates {
                arr[[idx, tile.as_usize()]] = 1.;
                if at_kan_select {
                    mask[tile.as_usize()] = true;
                }
            }
            if !at_kan_select {
                mask[42] = true;
            }
        }
        idx += 1;

        if cans.can_kakan {
            for tile in self.kakan_candidates {
                arr[[idx, tile.as_usize()]] = 1.;
                if at_kan_select {
                    mask[tile.as_usize()] = true;
                }
            }
            if !at_kan_select {
                mask[42] = true;
            }
        }
        idx += 1;

        if cans.can_tsumo_agari || cans.can_ron_agari {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[43] = true;
            }
        }
        idx += 1;

        if cans.can_ryukyoku {
            arr.slice_mut(s![idx, ..]).fill(1.);
            if !at_kan_select {
                mask[44] = true;
            }
        }
        idx += 1;

        assert_eq!(idx, OBS_SHAPE.0);
        (arr, mask)
    }
}
