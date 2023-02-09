use super::PlayerState;
use crate::consts::{obs_shape, ACTION_SPACE, MAX_VERSION};
use crate::state::item::KawaItem;
use crate::tile::Tile;
use crate::{tu8, tuz};
use std::num::NonZeroUsize;

use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;

struct ObsEncoderContext<'a> {
    state: &'a PlayerState,
    arr: Array2<f32>,
    mask: Array1<bool>,
    idx: usize,
    at_kan_select: bool,
    version: u32,
}

#[must_use]
struct IntegerEncoder {
    n: usize,
    cap: usize,
    one_hot: bool,
    rescale: bool,
    rbf_intervals: Option<NonZeroUsize>,
}

impl IntegerEncoder {
    const fn new(n: usize, cap: usize) -> Self {
        Self {
            n,
            cap,
            one_hot: false,
            rescale: false,
            rbf_intervals: None,
        }
    }
    const fn one_hot(mut self, v: bool) -> Self {
        self.one_hot = v;
        self
    }
    const fn rescale(mut self, v: bool) -> Self {
        self.rescale = v;
        self
    }
    const fn rbf_intervals(mut self, v: usize) -> Self {
        self.rbf_intervals = NonZeroUsize::new(v);
        self
    }

    fn encode(self, ctx: &mut ObsEncoderContext<'_>) {
        let n = self.n.min(self.cap);
        match ctx.version {
            1 => {
                ctx.arr.slice_mut(s![ctx.idx..ctx.idx + n, ..]).fill(1.);
                ctx.idx += self.cap;
            }
            2 | 3 => {
                debug_assert!(self.one_hot || self.rescale || self.rbf_intervals.is_some());

                if self.one_hot {
                    ctx.arr.slice_mut(s![ctx.idx + n, ..]).fill(1.);
                    ctx.idx += self.cap + 1;
                }
                if self.rescale {
                    let v = n as f32 / self.cap as f32;
                    ctx.arr.slice_mut(s![ctx.idx, ..]).fill(v);
                    ctx.idx += 1;
                }
                if let Some(intervals) = self.rbf_intervals.map(|v| v.get()) {
                    debug_assert!(intervals >= 3);
                    let interval_size = self.cap as f32 / intervals as f32;
                    for i in 1..intervals {
                        let x = self.n as f32; // the original value, not the clamped
                        let mu = i as f32 * interval_size;
                        let sigma = interval_size;
                        let v = (-(x - mu).powi(2) / (2. * sigma.powi(2))).exp();
                        ctx.arr.slice_mut(s![ctx.idx + i - 1, ..]).fill(v);
                    }
                    ctx.idx += intervals - 1;
                }
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> ObsEncoderContext<'a> {
    const SELF_KAWA_ITEM_CHANNELS: usize = 4;
    const KAWA_ITEM_CHANNELS: usize = 8;

    fn new(state: &'a PlayerState, version: u32, at_kan_select: bool) -> Self {
        assert!(version <= MAX_VERSION);
        let shape = obs_shape(version);
        let arr = Array2::zeros(shape);
        let mask = Array1::default(ACTION_SPACE);
        Self {
            state,
            arr,
            mask,
            idx: 0,
            at_kan_select,
            version,
        }
    }

    fn encode_obs(mut self) -> (Array2<f32>, Array1<bool>) {
        let state = self.state;
        let cans = state.last_cans;

        state
            .tehai
            .iter()
            .enumerate()
            .filter(|(_, &count)| count > 0)
            .for_each(|(tile_id, &count)| {
                let n = count as usize;
                self.arr
                    .slice_mut(s![self.idx..self.idx + n, tile_id])
                    .fill(1.);
            });
        self.idx += 4;

        state
            .akas_in_hand
            .into_iter()
            .enumerate()
            .filter(|&(_, has_it)| has_it)
            .for_each(|(i, _)| {
                self.arr.slice_mut(s![self.idx + i, ..]).fill(1.);
            });
        self.idx += 3;

        for &score in &state.scores {
            let v = score.clamp(0, 100_000) as f32 / 100_000.;
            self.arr.slice_mut(s![self.idx, ..]).fill(v);
            self.idx += 1;

            if matches!(self.version, 2 | 3) {
                IntegerEncoder::new(score as usize / 100, 500)
                    .rbf_intervals(10)
                    .encode(&mut self);
            }
        }

        let n = state.rank as usize;
        self.arr.slice_mut(s![self.idx + n, ..]).fill(1.);
        self.idx += 4;

        let n = state.kyoku as usize;
        match self.version {
            // for v1, this was a mistake, it actually only uses 3 channels.
            1 => self.arr.slice_mut(s![self.idx..self.idx + n, ..]).fill(1.),
            2 | 3 => self.arr.slice_mut(s![self.idx + n, ..]).fill(1.),
            _ => unreachable!(),
        }
        self.idx += 4;

        let cap = match self.version {
            1 => 10,
            2 | 3 => 6,
            _ => unreachable!(),
        };
        let n = state.honba as usize;
        IntegerEncoder::new(n, cap)
            .rbf_intervals(3)
            .encode(&mut self);
        let n = state.kyotaku as usize;
        IntegerEncoder::new(n, cap)
            .rbf_intervals(3)
            .encode(&mut self);

        self.arr[[self.idx, state.bakaze.as_usize()]] = 1.;
        self.arr[[self.idx + 1, state.jikaze.as_usize()]] = 1.;
        self.idx += 2;

        if matches!(self.version, 2 | 3) {
            let n = (state.bakaze.as_u8() - tu8!(E)).min(1) * 4 + state.kyoku;
            IntegerEncoder::new(n as usize, 7)
                .rescale(true)
                .encode(&mut self);
        }

        self.encode_tile_set(state.dora_indicators);

        state.kawa[0]
            .iter()
            .take(6)
            .for_each(|kawa_item| self.encode_self_kawa(kawa_item.as_ref()));
        self.idx += (6 - state.kawa[0].len().min(6)) * Self::SELF_KAWA_ITEM_CHANNELS;

        state.kawa[0]
            .iter()
            .rev()
            .take(18)
            .for_each(|kawa_item| self.encode_self_kawa(kawa_item.as_ref()));
        self.idx += (18 - state.kawa[0].len().min(18)) * Self::SELF_KAWA_ITEM_CHANNELS;

        let max_kawa_len = state.kawa.iter().map(|k| k.len()).max().unwrap();
        if self.version == 3 {
            for (turn, kawa_item) in state.kawa[0].iter().enumerate() {
                if let Some(kawa_item) = kawa_item {
                    let sutehai = kawa_item.sutehai;
                    let tid = sutehai.tile.deaka().as_usize();
                    let v = (-0.2 * (max_kawa_len - 1 - turn) as f32).exp();
                    self.arr[[self.idx, tid]] = v;
                }
            }
            self.idx += 1;
        }

        for player_kawa in &state.kawa[1..] {
            player_kawa
                .iter()
                .take(6)
                .for_each(|kawa_item| self.encode_kawa(kawa_item.as_ref()));
            self.idx += (6 - player_kawa.len().min(6)) * Self::KAWA_ITEM_CHANNELS;

            player_kawa
                .iter()
                .rev()
                .take(18)
                .for_each(|kawa_item| self.encode_kawa(kawa_item.as_ref()));
            self.idx += (18 - player_kawa.len().min(18)) * Self::KAWA_ITEM_CHANNELS;

            match self.version {
                2 => {
                    for (turn, kawa_item) in player_kawa.iter().flatten().enumerate() {
                        let row = (turn / 6).min(2);
                        let tid = kawa_item.sutehai.tile.deaka().as_usize();
                        self.arr[[self.idx + row, tid]] = 1.;
                        if kawa_item.sutehai.is_tedashi {
                            self.arr[[self.idx + 3 + row, tid]] = 1.;
                        }
                    }
                    self.idx += 6;
                }
                3 => {
                    for (turn, kawa_item) in player_kawa.iter().enumerate() {
                        if let Some(kawa_item) = kawa_item {
                            let sutehai = kawa_item.sutehai;
                            let tid = sutehai.tile.deaka().as_usize();
                            let v = (-0.2 * (max_kawa_len - 1 - turn) as f32).exp();
                            self.arr[[self.idx, tid]] = v;
                            if sutehai.is_tedashi {
                                self.arr[[self.idx + 1, tid]] = v;
                            }
                            if sutehai.is_riichi {
                                self.arr[[self.idx + 2, tid]] = v;
                            }
                        }
                    }
                    self.idx += 3;
                }
                _ => (),
            }
        }

        let v = state.tiles_left as f32 / 69.;
        self.arr.slice_mut(s![self.idx, ..]).fill(v);
        self.idx += 1;

        for count in state.doras_owned {
            IntegerEncoder::new(count as usize, 12)
                .rescale(true)
                .rbf_intervals(3)
                .encode(&mut self);
        }

        let doras_unseen = state.dora_indicators.len() as u8 * 4 + 3 - state.doras_seen;
        IntegerEncoder::new(doras_unseen as usize, 5 * 4 + 3)
            .rescale(true)
            .rbf_intervals(4)
            .encode(&mut self);

        for player_kawa_overview in &state.kawa_overview {
            self.encode_tile_set(player_kawa_overview.iter().copied());
        }

        for player_fuuro in &state.fuuro_overview {
            for f in player_fuuro {
                for tile in f {
                    let tile_id = tile.deaka().as_usize();
                    let i = (0..4)
                        .find(|&i| self.arr[[self.idx + i, tile_id]] == 0.)
                        .unwrap();
                    self.arr[[self.idx + i, tile_id]] = 1.;
                    // It is not possible to have more than one aka in a fuuro
                    // set, at least in tenhou rule, so we simply use one
                    // channel here.
                    if tile.is_aka() {
                        self.arr.slice_mut(s![self.idx + 4, ..]).fill(1.);
                    }
                }
                self.idx += 5;
            }
            self.idx += (4 - player_fuuro.len()) * 5;
        }

        for player_ankan in &state.ankan_overview {
            for tile in player_ankan {
                let tile_id = tile.as_usize();
                self.arr[[self.idx, tile_id]] = 1.;
            }
            self.idx += 1;
        }

        if matches!(self.version, 2 | 3) {
            for (tid, count) in state.tiles_seen.iter().copied().enumerate() {
                self.arr[[self.idx, tid]] = count as f32 / 4.;
            }
            self.idx += 1;

            for &player_last_tedashi in &state.last_tedashis[1..] {
                if let Some(sutehai) = player_last_tedashi {
                    let tile = sutehai.tile;
                    let tile_id = tile.deaka().as_usize();

                    self.arr[[self.idx, tile_id]] = 1.;
                    if tile.is_aka() {
                        self.arr.slice_mut(s![self.idx + 1, ..]).fill(1.);
                    }
                    if sutehai.is_dora {
                        self.arr.slice_mut(s![self.idx + 2, ..]).fill(1.);
                    }
                }
                self.idx += 3;
            }
            for &player_riichi_sutehai in &state.riichi_sutehais[1..] {
                if let Some(sutehai) = player_riichi_sutehai {
                    let tile = sutehai.tile;
                    let tile_id = tile.deaka().as_usize();

                    self.arr[[self.idx, tile_id]] = 1.;
                    if tile.is_aka() {
                        self.arr.slice_mut(s![self.idx + 1, ..]).fill(1.);
                    }
                    if sutehai.is_dora {
                        self.arr.slice_mut(s![self.idx + 2, ..]).fill(1.);
                    }
                }
                self.idx += 3;
            }
        }

        state.riichi_declared[1..]
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .for_each(|(i, _)| self.arr.slice_mut(s![self.idx + i, ..]).fill(1.));
        self.idx += 3;
        state.riichi_accepted[1..]
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .for_each(|(i, _)| self.arr.slice_mut(s![self.idx + i, ..]).fill(1.));
        self.idx += 3;

        state
            .waits
            .iter()
            .enumerate()
            .filter(|(_, &c)| c)
            .for_each(|(t, _)| self.arr[[self.idx, t]] = 1.);
        self.idx += 1;

        if state.at_furiten {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
        }
        self.idx += 1;

        let n = state.shanten as usize;
        IntegerEncoder::new(n, 6).one_hot(true).encode(&mut self);

        if state.riichi_accepted[0] {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
        }
        self.idx += 1;

        if self.at_kan_select {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
        }
        self.idx += 1;

        if cans.can_pass() {
            let tile = state
                .last_kawa_tile
                .expect("building chi/pon/daiminkan/ron feature without any kawa tile");
            let tile_id = tile.deaka().as_usize();

            self.arr[[self.idx, tile_id]] = 1.;
            if tile.is_aka() {
                self.arr.slice_mut(s![self.idx + 1, ..]).fill(1.);
            }
            if state.dora_factor[tile.deaka().as_usize()] > 0 {
                self.arr.slice_mut(s![self.idx + 2, ..]).fill(1.);
            }

            // pass
            if !self.at_kan_select {
                self.mask[ACTION_SPACE - 1] = true;
            } else if cans.can_daiminkan {
                self.mask[tile_id] = true;
            }
        }
        self.idx += 3;

        if cans.can_discard {
            state
                .discard_candidates_aka()
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
                    self.arr[[self.idx, deaka_t]] = 1.;
                    if !self.at_kan_select {
                        self.mask[t] = true;
                    }
                });

            state
                .keep_shanten_discards
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| self.arr[[self.idx + 1, t]] = 1.);
            state
                .next_shanten_discards
                .iter()
                .enumerate()
                .filter(|(_, &c)| c)
                .for_each(|(t, _)| self.arr[[self.idx + 2, t]] = 1.);

            if state.shanten <= 1 {
                state
                    .discard_candidates_with_unconditional_tenpai()
                    .iter()
                    .enumerate()
                    .filter(|(_, &c)| c)
                    .for_each(|(t, _)| self.arr[[self.idx + 3, t]] = 1.);
            }

            if state.riichi_declared[0] {
                self.arr.slice_mut(s![self.idx + 4, ..]).fill(1.);
            }
        }
        self.idx += 5;

        if cans.can_riichi {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[37] = true;
            }
        }
        self.idx += 1;

        if cans.can_chi_low {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[38] = true;
            }
        }
        if cans.can_chi_mid {
            self.arr.slice_mut(s![self.idx + 1, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[39] = true;
            }
        }
        if cans.can_chi_high {
            self.arr.slice_mut(s![self.idx + 2, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[40] = true;
            }
        }
        self.idx += 3;

        if cans.can_pon {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[41] = true;
            }
        }
        self.idx += 1;

        if cans.can_daiminkan {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[42] = true;
            }
        }
        self.idx += 1;

        if cans.can_ankan {
            for tile in state.ankan_candidates {
                self.arr[[self.idx, tile.as_usize()]] = 1.;
                if self.at_kan_select {
                    self.mask[tile.as_usize()] = true;
                }
            }
            if !self.at_kan_select {
                self.mask[42] = true;
            }
        }
        self.idx += 1;

        if cans.can_kakan {
            for tile in state.kakan_candidates {
                self.arr[[self.idx, tile.as_usize()]] = 1.;
                if self.at_kan_select {
                    self.mask[tile.as_usize()] = true;
                }
            }
            if !self.at_kan_select {
                self.mask[42] = true;
            }
        }
        self.idx += 1;

        if cans.can_agari() {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[43] = true;
            }
        }
        self.idx += 1;

        if cans.can_ryukyoku {
            self.arr.slice_mut(s![self.idx, ..]).fill(1.);
            if !self.at_kan_select {
                self.mask[44] = true;
            }
        }
        self.idx += 1;

        assert_eq!(self.idx, self.arr.shape()[0]);
        debug_assert!(self.arr.iter().all(|&v| (0. ..=1.).contains(&v)));
        (self.arr, self.mask)
    }

    fn encode_tile_set<I>(&mut self, tiles: I)
    where
        I: IntoIterator<Item = Tile>,
    {
        let mut counts = [0; 34];
        for tile in tiles {
            let tile_id = tile.deaka().as_usize();

            let i = &mut counts[tile_id];
            self.arr[[self.idx + *i, tile_id]] = 1.;
            *i += 1;

            if tile.is_aka() {
                let i = tile.as_usize() - tuz!(5mr);
                self.arr.slice_mut(s![self.idx + 4 + i, ..]).fill(1.);
            }
        }
        self.idx += 7;
    }

    fn encode_self_kawa(&mut self, item: Option<&KawaItem>) {
        if let Some(k) = item {
            for kan in k.kan {
                // deaka is required, it is possible for it to be an aka
                // (for example in Daiminkan and Kakan).
                let tile_id = kan.deaka().as_usize();
                self.arr[[self.idx, tile_id]] = 1.;
            }

            let sutehai = k.sutehai;
            let tile_id = sutehai.tile.deaka().as_usize();
            self.arr[[self.idx + 1, tile_id]] = 1.;
            if sutehai.tile.is_aka() {
                self.arr.slice_mut(s![self.idx + 2, ..]).fill(1.);
            }
            if sutehai.is_dora {
                self.arr.slice_mut(s![self.idx + 3, ..]).fill(1.);
            }
        }
        self.idx += Self::SELF_KAWA_ITEM_CHANNELS;
    }

    fn encode_kawa(&mut self, item: Option<&KawaItem>) {
        if let Some(k) = item {
            if let Some(cp) = &k.chi_pon {
                // Aka info of the chi/pon is not encoded in the kawa detail;
                // they are included in fuuro_overview instead.
                //
                // This is one-hot.
                let a = cp.consumed[0].deaka().as_usize();
                let b = cp.consumed[1].deaka().as_usize();
                let min = a.min(b);
                let max = a.max(b);
                self.arr[[self.idx, min]] = 1.;
                self.arr[[self.idx + 1, max]] = 1.;
            }

            for kan in k.kan {
                let tile_id = kan.deaka().as_usize();
                self.arr[[self.idx + 2, tile_id]] = 1.;
            }

            let sutehai = k.sutehai;
            let tile_id = sutehai.tile.deaka().as_usize();
            self.arr[[self.idx + 3, tile_id]] = 1.;
            if sutehai.tile.is_aka() {
                self.arr.slice_mut(s![self.idx + 4, ..]).fill(1.);
            }
            if sutehai.is_dora {
                self.arr.slice_mut(s![self.idx + 5, ..]).fill(1.);
            }
            if sutehai.is_tedashi {
                self.arr.slice_mut(s![self.idx + 6, ..]).fill(1.);
            }
            if sutehai.is_riichi {
                self.arr.slice_mut(s![self.idx + 7, ..]).fill(1.);
            }
        }
        self.idx += Self::KAWA_ITEM_CHANNELS;
    }
}

#[pymethods]
impl PlayerState {
    /// Returns `(obs, mask)`
    #[pyo3(name = "encode_obs")]
    fn encode_obs_py<'py>(
        &self,
        version: u32,
        at_kan_select: bool,
        py: Python<'py>,
    ) -> (&'py PyArray2<f32>, &'py PyArray1<bool>) {
        let (obs, mask) = self.encode_obs(version, at_kan_select);
        let obs = PyArray2::from_owned_array(py, obs);
        let mask = PyArray1::from_owned_array(py, mask);
        (obs, mask)
    }
}

impl PlayerState {
    /// Returns `(obs, mask)`
    #[must_use]
    pub fn encode_obs(&self, version: u32, at_kan_select: bool) -> (Array2<f32>, Array1<bool>) {
        ObsEncoderContext::new(self, version, at_kan_select).encode_obs()
    }
}
