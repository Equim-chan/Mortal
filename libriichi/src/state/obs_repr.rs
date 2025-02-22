use super::item::KawaItem;
use super::{PlayerState, SinglePlayerTables};
use crate::algo::sp::{Candidate, CandidateColumn};
use crate::array::Simple2DArray;
use crate::consts::{ACTION_SPACE, MAX_VERSION, obs_shape};
use crate::tile::Tile;
use crate::{tu8, tuz};
use std::num::NonZeroUsize;

use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;

const SELF_KAWA_ITEM_CHANNELS: usize = 4;
const KAWA_ITEM_CHANNELS: usize = 8;
const MAX_NUM_TURNS: usize = 17; // aka the actual practical `MAX_TSUMOS_LEFT`

struct ObsEncoderContext<'a> {
    state: &'a PlayerState,
    arr: Simple2DArray<34, f32>,
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
                ctx.arr.fill_rows(ctx.idx, n, 1.);
                ctx.idx += self.cap;
            }
            2 | 3 => {
                debug_assert!(self.one_hot || self.rescale || self.rbf_intervals.is_some());

                if self.one_hot {
                    ctx.arr.fill(ctx.idx + n, 1.);
                    ctx.idx += self.cap + 1;
                }
                if self.rescale {
                    let v = n as f32 / self.cap as f32;
                    ctx.arr.fill(ctx.idx, v);
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
                        ctx.arr.fill(ctx.idx + i - 1, v);
                    }
                    ctx.idx += intervals - 1;
                }
            }
            4 => {
                debug_assert!(self.one_hot || self.rescale);

                if self.one_hot {
                    ctx.arr.fill(ctx.idx + n, 1.);
                    ctx.idx += self.cap + 1;
                }
                if self.rescale {
                    let v = n as f32 / self.cap as f32;
                    ctx.arr.fill(ctx.idx, v);
                    ctx.idx += 1;
                }
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> ObsEncoderContext<'a> {
    fn new(state: &'a PlayerState, version: u32, at_kan_select: bool) -> Self {
        assert!(version <= MAX_VERSION);
        let shape = obs_shape(version);
        let arr = Simple2DArray::new(shape.0);
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
            .filter(|&(_, &count)| count > 0)
            .for_each(|(tile_id, &count)| {
                let n = count as usize;
                self.arr.assign_rows(self.idx, tile_id, n, 1.);
            });
        self.idx += 4;

        state
            .akas_in_hand
            .into_iter()
            .enumerate()
            .filter(|&(_, has_it)| has_it)
            .for_each(|(i, _)| self.arr.fill(self.idx + i, 1.));
        self.idx += 3;

        for &score in &state.scores {
            let v = score.clamp(0, 100_000) as f32 / 100_000.;
            self.arr.fill(self.idx, v);
            self.idx += 1;

            match self.version {
                2 | 3 => IntegerEncoder::new(score as usize / 100, 500)
                    .rbf_intervals(10)
                    .encode(&mut self),
                4 => {
                    let v = score.clamp(0, 30_000) as f32 / 30_000.;
                    self.arr.fill(self.idx, v);
                    self.idx += 1;
                }
                _ => (),
            }
        }

        let n = state.rank as usize;
        self.arr.fill(self.idx + n, 1.);
        self.idx += 4;

        let n = state.kyoku as usize;
        match self.version {
            // for v1, this was a mistake, it actually only uses 3 channels.
            1 => self.arr.fill_rows(self.idx, n, 1.),
            2 | 3 | 4 => self.arr.fill(self.idx + n, 1.),
            _ => unreachable!(),
        }
        self.idx += 4;

        let cap = match self.version {
            1 | 4 => 10,
            2 | 3 => 6,
            _ => unreachable!(),
        };
        let n = state.honba as usize;
        IntegerEncoder::new(n, cap)
            .rescale(self.version == 4)
            .rbf_intervals(3)
            .encode(&mut self);
        let n = state.kyotaku as usize;
        IntegerEncoder::new(n, cap)
            .rescale(self.version == 4)
            .rbf_intervals(3)
            .encode(&mut self);

        self.arr.assign(self.idx, state.bakaze.as_usize(), 1.);
        self.arr.assign(self.idx + 1, state.jikaze.as_usize(), 1.);
        self.idx += 2;

        if matches!(self.version, 2 | 3 | 4) {
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
        self.idx += (6 - state.kawa[0].len().min(6)) * SELF_KAWA_ITEM_CHANNELS;

        state.kawa[0]
            .iter()
            .rev()
            .take(18)
            .for_each(|kawa_item| self.encode_self_kawa(kawa_item.as_ref()));
        self.idx += (18 - state.kawa[0].len().min(18)) * SELF_KAWA_ITEM_CHANNELS;

        let max_kawa_len = state.kawa.iter().map(|k| k.len()).max().unwrap();
        if matches!(self.version, 3 | 4) {
            for (turn, kawa_item) in state.kawa[0].iter().enumerate() {
                if let Some(kawa_item) = kawa_item {
                    let sutehai = kawa_item.sutehai;
                    let tid = sutehai.tile.deaka().as_usize();
                    let v = (-0.2 * (max_kawa_len - 1 - turn) as f32).exp();
                    self.arr.assign(self.idx, tid, v);
                }
            }
            self.idx += 1;
        }

        for player_kawa in &state.kawa[1..] {
            player_kawa
                .iter()
                .take(6)
                .for_each(|kawa_item| self.encode_kawa(kawa_item.as_ref()));
            self.idx += (6 - player_kawa.len().min(6)) * KAWA_ITEM_CHANNELS;

            player_kawa
                .iter()
                .rev()
                .take(18)
                .for_each(|kawa_item| self.encode_kawa(kawa_item.as_ref()));
            self.idx += (18 - player_kawa.len().min(18)) * KAWA_ITEM_CHANNELS;

            match self.version {
                2 => {
                    for (turn, kawa_item) in player_kawa.iter().flatten().enumerate() {
                        let row = (turn / 6).min(2);
                        let tid = kawa_item.sutehai.tile.deaka().as_usize();
                        self.arr.assign(self.idx + row, tid, 1.);
                        if kawa_item.sutehai.is_tedashi {
                            self.arr.assign(self.idx + 3 + row, tid, 1.);
                        }
                    }
                    self.idx += 6;
                }
                3 | 4 => {
                    for (turn, kawa_item) in player_kawa.iter().enumerate() {
                        if let Some(kawa_item) = kawa_item {
                            let sutehai = kawa_item.sutehai;
                            let tid = sutehai.tile.deaka().as_usize();
                            let v = (-0.2 * (max_kawa_len - 1 - turn) as f32).exp();
                            self.arr.assign(self.idx, tid, v);
                            if sutehai.is_tedashi {
                                self.arr.assign(self.idx + 1, tid, v);
                            }
                            if sutehai.is_riichi {
                                self.arr.assign(self.idx + 2, tid, v);
                            }
                        }
                    }
                    self.idx += 3;
                }
                _ => (),
            }
        }

        let v = state.tiles_left as f32 / 69.;
        self.arr.fill(self.idx, v);
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
                        .find(|&i| self.arr.get(self.idx + i, tile_id) == 0.)
                        .unwrap();
                    self.arr.assign(self.idx + i, tile_id, 1.);
                    // It is not possible to have more than one aka in a fuuro
                    // set, at least in tenhou rule, so we simply use one
                    // channel here.
                    if tile.is_aka() {
                        self.arr.fill(self.idx + 4, 1.);
                    }
                }
                self.idx += 5;
            }
            self.idx += (4 - player_fuuro.len()) * 5;
        }

        for player_ankan in &state.ankan_overview {
            for tile in player_ankan {
                let tile_id = tile.as_usize();
                self.arr.assign(self.idx, tile_id, 1.);
            }
            self.idx += 1;
        }

        if matches!(self.version, 2 | 3 | 4) {
            for (tid, count) in state.tiles_seen.iter().copied().enumerate() {
                self.arr.assign(self.idx, tid, count as f32 / 4.);
            }
            self.idx += 1;

            for &player_last_tedashi in &state.last_tedashis[1..] {
                if let Some(sutehai) = player_last_tedashi {
                    let tile = sutehai.tile;
                    let tile_id = tile.deaka().as_usize();

                    self.arr.assign(self.idx, tile_id, 1.);
                    if tile.is_aka() {
                        self.arr.fill(self.idx + 1, 1.);
                    }
                    if sutehai.is_dora {
                        self.arr.fill(self.idx + 2, 1.);
                    }
                }
                self.idx += 3;
            }
            for &player_riichi_sutehai in &state.riichi_sutehais[1..] {
                if let Some(sutehai) = player_riichi_sutehai {
                    let tile = sutehai.tile;
                    let tile_id = tile.deaka().as_usize();

                    self.arr.assign(self.idx, tile_id, 1.);
                    if tile.is_aka() {
                        self.arr.fill(self.idx + 1, 1.);
                    }
                    if sutehai.is_dora {
                        self.arr.fill(self.idx + 2, 1.);
                    }
                }
                self.idx += 3;
            }
        }

        state.riichi_declared[1..]
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .for_each(|(i, _)| self.arr.fill(self.idx + i, 1.));
        self.idx += 3;
        state.riichi_accepted[1..]
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .for_each(|(i, _)| self.arr.fill(self.idx + i, 1.));
        self.idx += 3;

        state
            .waits
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c)
            .for_each(|(t, _)| self.arr.assign(self.idx, t, 1.));
        self.idx += 1;

        if state.at_furiten {
            self.arr.fill(self.idx, 1.);
        }
        self.idx += 1;

        let n = state.shanten as usize;
        IntegerEncoder::new(n, 6).one_hot(true).encode(&mut self);

        if state.riichi_accepted[0] {
            self.arr.fill(self.idx, 1.);
        }
        self.idx += 1;

        if self.at_kan_select {
            self.arr.fill(self.idx, 1.);
        }
        self.idx += 1;

        if cans.can_pass() {
            let tile = state
                .last_kawa_tile
                .expect("building chi/pon/daiminkan/ron feature without any kawa tile");
            let tile_id = tile.deaka().as_usize();

            self.arr.assign(self.idx, tile_id, 1.);
            if tile.is_aka() {
                self.arr.fill(self.idx + 1, 1.);
            }
            if state.dora_factor[tile.deaka().as_usize()] > 0 {
                self.arr.fill(self.idx + 2, 1.);
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
                .filter(|&(_, &c)| c)
                .for_each(|(t, _)| {
                    let deaka_t = match t as u8 {
                        tu8!(5mr) => tuz!(5m),
                        tu8!(5pr) => tuz!(5p),
                        tu8!(5sr) => tuz!(5s),
                        _ => t,
                    };
                    self.arr.assign(self.idx, deaka_t, 1.);
                    if !self.at_kan_select {
                        self.mask[t] = true;
                    }
                });

            state
                .keep_shanten_discards
                .iter()
                .enumerate()
                .filter(|&(_, &c)| c)
                .for_each(|(t, _)| self.arr.assign(self.idx + 1, t, 1.));
            state
                .next_shanten_discards
                .iter()
                .enumerate()
                .filter(|&(_, &c)| c)
                .for_each(|(t, _)| self.arr.assign(self.idx + 2, t, 1.));

            if state.shanten <= 1 {
                state
                    .discard_candidates_with_unconditional_tenpai()
                    .iter()
                    .enumerate()
                    .filter(|&(_, &c)| c)
                    .for_each(|(t, _)| self.arr.assign(self.idx + 3, t, 1.));
            }

            if state.riichi_declared[0] {
                self.arr.fill(self.idx + 4, 1.);
            }
        }
        self.idx += 5;

        if cans.can_riichi {
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[37] = true;
            }
        }
        self.idx += 1;

        if cans.can_chi_low {
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[38] = true;
            }
        }
        if cans.can_chi_mid {
            self.arr.fill(self.idx + 1, 1.);
            if !self.at_kan_select {
                self.mask[39] = true;
            }
        }
        if cans.can_chi_high {
            self.arr.fill(self.idx + 2, 1.);
            if !self.at_kan_select {
                self.mask[40] = true;
            }
        }
        self.idx += 3;

        if cans.can_pon {
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[41] = true;
            }
        }
        self.idx += 1;

        if cans.can_daiminkan {
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[42] = true;
            }
        }
        self.idx += 1;

        if cans.can_ankan {
            for tile in state.ankan_candidates {
                self.arr.assign(self.idx, tile.as_usize(), 1.);
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
                self.arr.assign(self.idx, tile.as_usize(), 1.);
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
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[43] = true;
            }
        }
        self.idx += 1;

        if cans.can_ryukyoku {
            self.arr.fill(self.idx, 1.);
            if !self.at_kan_select {
                self.mask[44] = true;
            }
        }
        self.idx += 1;

        if self.version == 4 {
            if let Ok(SinglePlayerTables { max_ev_table }) = state.single_player_tables() {
                // Get the max EV from the table that maximizes EV, which should
                // be the global max EV.
                //
                // `max_ev_table` is already sorted.
                let max_ev = max_ev_table
                    .first()
                    .and_then(|c| c.exp_values.first().copied())
                    .unwrap_or_default();
                self.encode_ev(max_ev);

                // Encode required tiles.
                if cans.can_discard {
                    for candidate in &max_ev_table {
                        let discard_tid = candidate.tile.deaka().as_usize();
                        for r in &candidate.required_tiles {
                            let required_tid = r.tile.deaka().as_usize();
                            if candidate.shanten_down {
                                self.arr
                                    .assign(self.idx + 34 + discard_tid, required_tid, 1.);
                            } else {
                                self.arr.assign(self.idx + discard_tid, required_tid, 1.);
                            }
                        }
                    }
                    self.idx += 2 * 34;

                    let max_required_tiles_tid = max_ev_table
                        .iter()
                        .max_by(|l, r| l.cmp(r, CandidateColumn::NotShantenDown))
                        .unwrap()
                        .tile
                        .deaka()
                        .as_usize();
                    self.arr.assign(self.idx, max_required_tiles_tid, 1.);
                    self.idx += 2;
                } else {
                    self.idx += 2 * 34 + 1;
                    for r in &max_ev_table[0].required_tiles {
                        let required_tid = r.tile.deaka().as_usize();
                        self.arr.assign(self.idx, required_tid, 1.);
                    }
                    self.idx += 1;
                }

                let ev_scale = if max_ev < 1. { 0. } else { 1. / max_ev };
                self.encode_sp_table(max_ev_table, cans.can_discard, ev_scale);
            } else {
                // Use the minimal tsumo agari point as the max EV. It is
                // minimal because we assume no uradora.
                let min_tsumo_agari = state
                    .agari_points(cans.can_ron_agari, &[])
                    .map(|p| p.tsumo_total(state.is_oya()) as f32)
                    .unwrap_or_default();
                self.encode_ev(min_tsumo_agari);

                // Skip everything else.
                self.idx += 2 * 34 + 2 + 3 * MAX_NUM_TURNS;
            }
        }

        assert_eq!(self.idx, self.arr.rows());
        let arr = self.arr.build();
        debug_assert!(arr.iter().all(|&v| (0. ..=1.).contains(&v)));
        (arr, self.mask)
    }

    fn encode_ev(&mut self, value: f32) {
        let v = value.clamp(0., 100_000.) / 100_000.;
        self.arr.fill(self.idx, v);
        let v = value.clamp(0., 30_000.) / 30_000.;
        self.arr.fill(self.idx + 1, v);
        self.idx += 2;
    }

    // discard table: 3 * MAX_NUM_TURNS
    // tsumo table: 3 * MAX_NUM_TURNS
    // best ev discard: 1
    // best win prob discard: 1
    fn encode_sp_table(&mut self, candidates: Vec<Candidate>, can_discard: bool, ev_scale: f32) {
        let Some(first) = candidates
            .first()
            .filter(|c| c.tenpai_probs.first().is_some_and(|&p| p > 0.))
        else {
            // Simply do nothing when probs aren't calculated at all (when
            // shanten >= 4) or are all zero.
            self.idx += 3 * MAX_NUM_TURNS;
            return;
        };

        if can_discard {
            for candidate in candidates {
                let tid = candidate.tile.deaka().as_usize();
                for (turn, ((&tenpai_prob, &win_prob), &ev)) in candidate
                    .tenpai_probs
                    .iter()
                    .take_while(|&&p| p > 0.)
                    .zip(&candidate.win_probs)
                    .zip(&candidate.exp_values)
                    .enumerate()
                {
                    let mut idx = self.idx + turn;
                    self.arr.assign(idx, tid, tenpai_prob);
                    idx += MAX_NUM_TURNS;
                    self.arr.assign(idx, tid, win_prob);
                    idx += MAX_NUM_TURNS;
                    self.arr.assign(idx, tid, (ev * ev_scale).min(1.));
                }
            }
        } else {
            for (turn, ((&tenpai_prob, &win_prob), &ev)) in first
                .tenpai_probs
                .iter()
                .take_while(|&&p| p > 0.)
                .zip(&first.win_probs)
                .zip(&first.exp_values)
                .enumerate()
            {
                let mut idx = self.idx + turn;
                self.arr.fill(idx, tenpai_prob);
                idx += MAX_NUM_TURNS;
                self.arr.fill(idx, win_prob);
                idx += MAX_NUM_TURNS;
                self.arr.fill(idx, (ev * ev_scale).min(1.));
            }
        }
        self.idx += 3 * MAX_NUM_TURNS;
    }

    fn encode_tile_set<I>(&mut self, tiles: I)
    where
        I: IntoIterator<Item = Tile>,
    {
        let mut counts = [0; 34];
        for tile in tiles {
            let tile_id = tile.deaka().as_usize();

            let i = &mut counts[tile_id];
            self.arr.assign(self.idx + *i, tile_id, 1.);
            *i += 1;

            if tile.is_aka() {
                let i = tile.as_usize() - tuz!(5mr);
                self.arr.fill(self.idx + 4 + i, 1.);
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
                self.arr.assign(self.idx, tile_id, 1.);
            }

            let sutehai = k.sutehai;
            let tile_id = sutehai.tile.deaka().as_usize();
            self.arr.assign(self.idx + 1, tile_id, 1.);
            if sutehai.tile.is_aka() {
                self.arr.fill(self.idx + 2, 1.);
            }
            if sutehai.is_dora {
                self.arr.fill(self.idx + 3, 1.);
            }
        }
        self.idx += SELF_KAWA_ITEM_CHANNELS;
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
                self.arr.assign(self.idx, min, 1.);
                self.arr.assign(self.idx + 1, max, 1.);
            }

            for kan in k.kan {
                let tile_id = kan.deaka().as_usize();
                self.arr.assign(self.idx + 2, tile_id, 1.);
            }

            let sutehai = k.sutehai;
            let tile_id = sutehai.tile.deaka().as_usize();
            self.arr.assign(self.idx + 3, tile_id, 1.);
            if sutehai.tile.is_aka() {
                self.arr.fill(self.idx + 4, 1.);
            }
            if sutehai.is_dora {
                self.arr.fill(self.idx + 5, 1.);
            }
            if sutehai.is_tedashi {
                self.arr.fill(self.idx + 6, 1.);
            }
            if sutehai.is_riichi {
                self.arr.fill(self.idx + 7, 1.);
            }
        }
        self.idx += KAWA_ITEM_CHANNELS;
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
    ) -> (Bound<'py, PyArray2<f32>>, Bound<'py, PyArray1<bool>>) {
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
