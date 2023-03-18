use super::result::KyokuResult;
use crate::consts::oracle_obs_shape;
use crate::mjai::{Event, EventExt};
use crate::state::PlayerState;
use crate::tile::Tile;
use crate::vec_ops::vec_add_assign;
use crate::{matches_tu8, must_tile, t, tu8};
use std::array;
use std::convert::TryInto;
use std::mem;

use anyhow::{bail, Context, Result};
use derivative::Derivative;
use ndarray::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use sha3::{Digest, Sha3_256};
use tinyvec::ArrayVec;

/// The fields are all pub on purpose so the caller will be able to set the
/// yama, doras, scores directly.
///
/// Other than what is mentioned below, everything else is identical to Tenhou's
/// Rule.
///
/// 1. No triple-ron ryukyoku.
/// 2. Tenhou (the yaku) and chihou do not accumulate with other yakus; they are
///    always 1x yakuman.
#[derive(Debug, Default)]
pub struct Board {
    /// Counts from 0
    pub kyoku: u8,
    pub honba: u8,
    /// Does not effect the kyoku seed
    pub kyotaku: u8,
    /// [25000; 4]
    pub scores: [i32; 4],

    pub haipai: [[Tile; 13]; 4],
    /// Goes backward (pop)
    pub yama: Vec<Tile>,
    /// Goes backward (pop)
    pub rinshan: Vec<Tile>,
    /// Goes backward (pop)
    pub dora_indicators: Vec<Tile>,
    /// Goes forward (iter)
    pub ura_indicators: Vec<Tile>,
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct BoardState {
    board: Board,
    // Absolute seat, with the oya of E1 always being 0
    oya: u8,
    player_states: [PlayerState; 4],

    can_renchan: bool,
    has_hora: bool,
    has_abortive_ryukyoku: bool,
    kyoku_deltas: [i32; 4],

    #[derivative(Default(value = "70"))]
    tiles_left: u8,
    tsumo_actor: u8,
    // Just a fancy bool
    deal_from_rinshan: Option<()>,
    need_new_dora_at_discard: Option<()>,
    need_new_dora_at_tsumo: Option<()>,
    riichi_to_be_accepted: Option<u8>,
    #[derivative(Default(value = "[true; 4]"))]
    can_nagashi_mangan: [bool; 4],
    #[derivative(Default(value = "true"))]
    can_four_wind: bool,
    four_wind_tile: Option<Tile>,
    accepted_riichis: u8,
    kans: u8,
    check_four_kan: bool,
    paos: [Option<u8>; 4],

    log: Vec<EventExt>,

    // For oracle_obs only
    dora_indicators_full: Vec<Tile>,
}

pub struct AgentContext<'a> {
    pub player_states: &'a [PlayerState; 4],
    pub log: &'a [EventExt],
}

#[derive(Clone, Copy)]
pub enum Poll {
    InGame,
    End,
}

impl Board {
    pub fn init_from_seed(&mut self, game_seed: (u64, u64)) {
        let (nonce, key) = game_seed;
        let kyoku_seed = Sha3_256::new()
            .chain_update(nonce.to_le_bytes())
            .chain_update(key.to_le_bytes())
            .chain_update([self.kyoku, self.honba])
            .finalize()
            .try_into()
            .unwrap();
        let mut rng = ChaCha12Rng::from_seed(kyoku_seed);
        let mut seq = UNSHUFFLED;
        seq.shuffle(&mut rng);

        self.haipai = array::from_fn(|i| seq[i * 13..(i + 1) * 13].try_into().unwrap());
        let mut idx = 13 * 4;

        self.rinshan = seq[idx..idx + 4].to_vec();
        idx += 4;
        self.dora_indicators = seq[idx..idx + 5].to_vec();
        idx += 5;
        self.ura_indicators = seq[idx..idx + 5].to_vec();
        idx += 5;
        self.yama = seq[idx..idx + 70].to_vec();
        idx += 70;
        assert_eq!(idx, seq.len());
    }

    pub fn into_state(self) -> BoardState {
        let oya = self.kyoku % 4;
        let dora_indicators_full = self.dora_indicators.clone();

        BoardState {
            board: self,
            oya,
            player_states: [
                PlayerState::new(0),
                PlayerState::new(1),
                PlayerState::new(2),
                PlayerState::new(3),
            ],
            dora_indicators_full,
            ..Default::default()
        }
    }
}

impl BoardState {
    pub fn poll(&mut self, mut reactions: [EventExt; 4]) -> Result<Poll> {
        loop {
            let poll = self.step(&reactions)?;
            match poll {
                Poll::InGame => {
                    if self.player_states.iter().any(|c| c.last_cans().can_act()) {
                        return Ok(poll);
                    }
                }
                Poll::End => {
                    self.add_log_no_meta(Event::EndKyoku);
                    vec_add_assign(&mut self.board.scores, &self.kyoku_deltas);
                    if self.has_abortive_ryukyoku {
                        self.can_renchan = true;
                    }
                    return Ok(poll);
                }
            };
            reactions = Default::default();
        }
    }

    #[inline]
    pub fn agent_context(&self) -> AgentContext<'_> {
        AgentContext {
            player_states: &self.player_states,
            log: &self.log,
        }
    }

    #[inline]
    pub const fn end(&self) -> KyokuResult {
        KyokuResult {
            kyoku: self.board.kyoku,
            honba: self.board.honba,
            can_renchan: self.can_renchan,
            has_hora: self.has_hora,
            has_abortive_ryukyoku: self.has_abortive_ryukyoku,
            kyotaku_left: self.board.kyotaku,
            scores: self.board.scores,
        }
    }

    #[inline]
    pub fn take_log(&mut self) -> Vec<EventExt> {
        mem::take(&mut self.log)
    }

    #[inline]
    fn add_log(&mut self, ev: EventExt) {
        self.log.push(ev);
    }

    #[inline]
    fn add_log_no_meta(&mut self, ev: Event) {
        self.log.push(EventExt::no_meta(ev));
    }

    #[inline]
    fn broadcast(&mut self, ev: &Event) {
        for s in &mut self.player_states {
            s.update(ev);
        }
    }

    fn haipai(&mut self) -> Result<()> {
        let bakaze = must_tile!(tu8!(E) + self.board.kyoku / 4);
        let start_kyoku = Event::StartKyoku {
            bakaze,
            dora_marker: self
                .board
                .dora_indicators
                .pop()
                .context("insufficient dora indicators")?,
            kyoku: self.oya + 1,
            honba: self.board.honba,
            kyotaku: self.board.kyotaku,
            oya: self.oya,
            scores: self.board.scores,
            tehais: self.board.haipai,
        };
        self.broadcast(&start_kyoku);
        self.add_log_no_meta(start_kyoku);

        let tile = self
            .board
            .yama
            .pop()
            .context("invalid yama: empty at init")?;
        self.tiles_left -= 1;
        let first_tsumo = Event::Tsumo {
            actor: self.oya,
            pai: tile,
        };
        self.broadcast(&first_tsumo);
        self.add_log_no_meta(first_tsumo);

        Ok(())
    }

    fn exhaustive_ryukyoku(&mut self) {
        let mut deltas = [0; 4];
        self.can_renchan = self.player_states[self.oya as usize].shanten() == 0;

        let mut has_nagashi_mangan = false;
        self.can_nagashi_mangan
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .map(|(i, _)| i)
            .for_each(|i| {
                has_nagashi_mangan = true;
                if i as u8 == self.oya {
                    let mut dod = [-4000; 4];
                    dod[i] = 12000;
                    vec_add_assign(&mut deltas, &dod);
                } else {
                    let mut dod = [-2000; 4];
                    dod[i] = 8000;
                    dod[self.oya as usize] = -4000;
                    vec_add_assign(&mut deltas, &dod);
                };
            });

        if !has_nagashi_mangan {
            let tenpai_actors: ArrayVec<[_; 4]> = self
                .player_states
                .iter()
                .enumerate()
                .filter(|(_, s)| s.shanten() == 0)
                .map(|(i, _)| i)
                .collect();

            let (plus, minus) = match tenpai_actors.len() {
                1 => (3000, -1000),
                2 => (1500, -1500),
                3 => (1000, -3000),
                // 0 | 4
                _ => (0, 0),
            };
            if plus > 0 {
                let mut dod = [minus; 4];
                tenpai_actors.into_iter().for_each(|i| dod[i] = plus);
                vec_add_assign(&mut deltas, &dod);
            }
        }

        vec_add_assign(&mut self.kyoku_deltas, &deltas);
        let ryukyoku = Event::Ryukyoku {
            deltas: Some(deltas),
        };
        self.add_log_no_meta(ryukyoku);
        // no need to broadcast
    }

    fn update_nagashi_mangan_and_four_wind(&mut self, ev: &Event) {
        match *ev {
            Event::Dahai { actor, pai, .. } if !pai.is_yaokyuu() => {
                self.can_nagashi_mangan[actor as usize] = false;
            }
            Event::Chi { target, .. }
            | Event::Pon { target, .. }
            | Event::Daiminkan { target, .. } => {
                self.can_nagashi_mangan[target as usize] = false;
                self.can_four_wind = false;
            }
            Event::Ankan { .. } => {
                self.can_four_wind = false;
            }
            _ => (),
        };
    }

    fn check_four_wind(&mut self, pai: Tile) -> Result<bool> {
        if !matches_tu8!(pai.as_u8(), E | S | W | N) {
            self.can_four_wind = false;
        } else if self.player_states[self.tsumo_actor as usize].can_w_riichi() {
            if let Some(tile) = self.four_wind_tile {
                // compare if the tile is equal to the first
                // wind
                self.can_four_wind = tile == pai;
            } else {
                // the very first discard and it is a wind,
                // record the wind
                self.four_wind_tile = Some(pai);
            }
        } else if let Some(tile) = self.four_wind_tile {
            // check if the first jun is just over and the last
            // discarded wind is still the same as the previous
            if tile == pai {
                return Ok(true);
            }
            // do not bother checking it again
            self.can_four_wind = false;
        } else {
            bail!("unexpected state when calculating 四風連打");
        }

        Ok(false)
    }

    fn check_riichi_accepted(&mut self) {
        if let Some(actor) = self.riichi_to_be_accepted.take() {
            let riichi_accepted = Event::ReachAccepted { actor };
            self.broadcast(&riichi_accepted);
            self.add_log_no_meta(riichi_accepted);
            self.board.scores[actor as usize] -= 1000;
            self.board.kyotaku += 1;
            self.accepted_riichis += 1;
        }
    }

    fn add_new_dora(&mut self) -> Result<()> {
        let dora = self
            .board
            .dora_indicators
            .pop()
            .context("illegal kan: already 4 kans and this is the 5th")?;
        let dora_ev = Event::Dora { dora_marker: dora };
        self.broadcast(&dora_ev);
        self.add_log_no_meta(dora_ev);

        Ok(())
    }

    fn handle_hora(
        &mut self,
        single_actor: u8,
        single_target: u8,
        reactions: &[EventExt; 4],
    ) -> Result<()> {
        self.has_hora = true;

        let is_ron = single_actor != single_target;
        let mut honba_left = self.board.honba as i32; // mut in case of multi-ron
        let mut kyotaku_point = self.board.kyotaku as i32 * 1000; // ditto
        self.board.kyotaku = 0; // Unlike honba, kyotaku in self will be cleared

        // Let the states get their agari points provided with our ura
        // indicators.
        let ura_indicators =
            self.board.ura_indicators[..5 - self.board.dora_indicators.len()].to_vec();
        let points = reactions
            .iter()
            .map(|ev| match ev.event {
                Event::Hora { actor, .. } => {
                    self.can_renchan |= actor == self.oya;
                    let point =
                        self.player_states[actor as usize].agari_points(is_ron, &ura_indicators);
                    Some(point).transpose()
                }
                _ => Ok(None),
            })
            .collect::<Result<Vec<_>>>()?;

        if is_ron {
            // Multi-ron will be handled
            points
                .into_iter()
                .enumerate()
                .cycle()
                .skip(single_target as usize + 1)
                .take(3)
                .filter_map(|(actor, v)| v.map(|point| (actor, point)))
                .for_each(|(actor, point)| {
                    let mut deltas = [0; 4];
                    if let Some(pao_target) = self.paos[actor] {
                        // As per [Tenhou's rule](https://tenhou.net/man/#RULE):
                        //
                        // > 複合役満を含む得点を、ツモ＝全額・ロン＝折半で支払
                        // > う。積み棒は包。
                        deltas[pao_target as usize] = -point.ron / 2 - honba_left * 300;
                        deltas[single_target as usize] -= point.ron / 2; // they may be the same person
                    } else {
                        deltas[single_target as usize] = -point.ron - honba_left * 300;
                    }
                    deltas[actor] = point.ron + kyotaku_point + honba_left * 300;

                    kyotaku_point = 0;
                    honba_left = 0;

                    vec_add_assign(&mut self.kyoku_deltas, &deltas);
                    let ura_markers = self.player_states[actor]
                        .self_riichi_accepted()
                        .then(|| ura_indicators.clone())
                        .unwrap_or_default();

                    let hora = Event::Hora {
                        actor: actor as u8,
                        target: single_target,
                        deltas: Some(deltas),
                        ura_markers: Some(ura_markers),
                    };
                    self.add_log_no_meta(hora);
                    // No need to broadcast
                });
            return Ok(());
        }

        let point = points[single_actor as usize].unwrap();
        let mut deltas = [0; 4];
        if let Some(pao_target) = self.paos[single_actor as usize] {
            // For pao to happen, the agari must have at least 1 yakuman so ron
            // point and sum of tsumo point should be equal.
            deltas[pao_target as usize] = -point.ron - honba_left * 300;
        } else {
            deltas.fill(-point.tsumo_ko - honba_left * 100);
            if single_actor != self.oya {
                deltas[self.oya as usize] = -point.tsumo_oya - honba_left * 100;
            }
        };
        deltas[single_actor as usize] =
            point.tsumo_total(single_actor == self.oya) + kyotaku_point + honba_left * 300;

        vec_add_assign(&mut self.kyoku_deltas, &deltas);
        let ura_markers = self.player_states[single_actor as usize]
            .self_riichi_accepted()
            .then_some(ura_indicators)
            .unwrap_or_default();

        let hora = Event::Hora {
            actor: single_actor,
            target: single_target,
            deltas: Some(deltas),
            ura_markers: Some(ura_markers),
        };
        self.add_log_no_meta(hora);
        // No need to broadcast

        Ok(())
    }

    fn update_paos(&mut self, ev: &Event) {
        match *ev {
            Event::Pon {
                target, actor, pai, ..
            }
            | Event::Daiminkan {
                target, actor, pai, ..
            } if pai.is_jihai() => {
                let mut jihais = 0_u8;
                self.player_states[actor as usize]
                    .pons()
                    .iter()
                    .chain(self.player_states[actor as usize].minkans())
                    .copied()
                    .filter(|&t| t >= tu8!(E))
                    .for_each(|t| jihais |= 1 << (t - tu8!(E)));
                let daisanein_confirmed = (jihais & 0b1110000) == 0b1110000;
                let daisuushi_confirmed = (jihais & 0b0001111) == 0b0001111;
                if daisanein_confirmed && matches_tu8!(pai.as_u8(), P | F | C)
                    || daisuushi_confirmed && matches_tu8!(pai.as_u8(), E | S | W | N)
                {
                    self.paos[actor as usize] = Some(target);
                }
            }
            _ => (),
        }
    }

    #[inline]
    fn abortive_ryukyoku(&mut self) {
        let ryukyoku = Event::Ryukyoku {
            deltas: Some([0; 4]),
        };
        self.add_log_no_meta(ryukyoku);
        self.has_abortive_ryukyoku = true;
        // No need to broadcast
    }

    fn step(&mut self, reactions: &[EventExt; 4]) -> Result<Poll> {
        if self.tiles_left == 70 {
            self.haipai()?;
            return Ok(Poll::InGame);
        }

        if self.accepted_riichis == 4 {
            // 四家立直
            self.abortive_ryukyoku();
            return Ok(Poll::End);
        }

        // Validate reactions
        for (actor, ev) in reactions.iter().enumerate() {
            self.player_states[actor]
                .validate_reaction(&ev.event)
                .with_context(|| {
                    format!(
                        "invalid action: {ev:?}\nstate:\n{}",
                        self.player_states[actor].brief_info(),
                    )
                })?;
        }

        let ev = reactions
            .iter()
            .min_by_key(|ev| match ev.event {
                Event::Hora { .. } => 0,
                Event::Daiminkan { .. } | Event::Pon { .. } => 1,
                Event::None => 3,
                _ => 2,
            })
            .unwrap(); // Unwrap is safe because it is proven non-empty

        if self.check_four_kan && !matches!(ev.event, Event::Hora { .. }) {
            // 四槓散了
            self.abortive_ryukyoku();
            return Ok(Poll::End);
        }

        self.update_nagashi_mangan_and_four_wind(&ev.event);

        match ev.event {
            Event::None => {
                if self.tiles_left == 0 {
                    self.exhaustive_ryukyoku();
                    return Ok(Poll::End);
                }
                self.check_riichi_accepted();

                let tile = if self.deal_from_rinshan.take().is_some() {
                    self.board
                        .rinshan
                        .pop()
                        .context("illegal kan: already 4 kans and this is the 5th")?
                } else {
                    self.board.yama.pop().with_context(|| {
                        format!("tiles left > 0 ({}) but yama is empty", self.tiles_left)
                    })?
                };
                self.tiles_left -= 1;
                let tsumo = Event::Tsumo {
                    actor: self.tsumo_actor,
                    pai: tile,
                };

                // This is for Kakan only because chankan is possible until an
                // actual tsumo.
                if self.need_new_dora_at_tsumo.take().is_some() {
                    self.add_new_dora()?;
                }

                self.broadcast(&tsumo);
                self.add_log_no_meta(tsumo);
            }

            Event::Dahai { actor, pai, .. } => {
                if self.need_new_dora_at_discard.take().is_some() {
                    self.add_new_dora()?;
                }

                self.broadcast(&ev.event);
                self.add_log(ev.clone());
                self.tsumo_actor = (actor + 1) % 4;

                // 四風連打
                if self.can_four_wind && self.check_four_wind(pai)? {
                    self.abortive_ryukyoku();
                    return Ok(Poll::End);
                }

                if self.kans == 4 && self.player_states.iter().all(|s| s.kans_count() < 4) {
                    // 四槓散了
                    self.check_four_kan = true;
                }
            }

            Event::Chi { .. } | Event::Pon { .. } => {
                self.check_riichi_accepted();
                self.broadcast(&ev.event);
                self.add_log(ev.clone());
            }

            Event::Ankan { actor, .. } => {
                // For continuous kan
                if self.need_new_dora_at_discard.take().is_some() {
                    self.add_new_dora()?;
                }

                self.broadcast(&ev.event);
                self.add_log(ev.clone());

                // Immediately add new dora
                self.add_new_dora()?;

                self.tsumo_actor = actor;
                self.deal_from_rinshan = Some(());
                self.kans += 1;
            }

            Event::Daiminkan { actor, .. } | Event::Kakan { actor, .. } => {
                // For Kakan only, do not `.take()` it.
                if self.need_new_dora_at_discard.is_some() {
                    self.need_new_dora_at_tsumo = Some(());
                }

                // For Daiminkan only
                self.check_riichi_accepted();

                self.broadcast(&ev.event);
                self.add_log(ev.clone());

                self.need_new_dora_at_discard = Some(());

                self.tsumo_actor = actor;
                self.deal_from_rinshan = Some(());
                self.kans += 1;
            }

            Event::Reach { actor } => {
                self.broadcast(&ev.event);
                self.add_log(ev.clone());
                self.riichi_to_be_accepted = Some(actor);
            }

            Event::Hora { actor, target, .. } => {
                self.handle_hora(actor, target, reactions)?;
                return Ok(Poll::End);
            }

            Event::Ryukyoku { .. } => {
                // 九種九牌
                self.abortive_ryukyoku();
                return Ok(Poll::End);
            }

            _ => {
                bail!("unexpected event: {:?}", ev.event);
            }
        };

        // The pao check cannot be done before the current event (Pon or
        // Daiminkan) gets processed because it requires to read `.pons()` and
        // `.minkans()`.
        self.update_paos(&ev.event);

        Ok(Poll::InGame)
    }

    pub fn encode_oracle_obs(&self, perspective: u8, version: u32) -> Array2<f32> {
        let shape = oracle_obs_shape(version);
        let mut arr = Array2::zeros(shape);
        let mut idx = 0;

        self.player_states
            .iter()
            .cycle()
            .skip(perspective as usize + 1)
            .take(3)
            .for_each(|state| {
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
            });

        let mut encode_tile = |idx: usize, tile: Tile| {
            let tile_id = tile.deaka().as_usize();
            arr[[idx, tile_id]] = 1.;
            if tile.is_aka() {
                arr.slice_mut(s![idx + 1, ..]).fill(1.);
            }
        };

        self.board
            .yama
            .iter()
            .copied()
            .rev()
            .take(self.tiles_left as usize)
            .for_each(|tile| {
                encode_tile(idx, tile);
                idx += 2;
            });
        idx += (69 - self.tiles_left as usize) * 2;

        self.board.rinshan.iter().copied().rev().for_each(|tile| {
            encode_tile(idx, tile);
            idx += 2;
        });
        idx += (4 - self.board.rinshan.len()) * 2;

        self.dora_indicators_full
            .iter()
            .copied()
            .rev()
            .for_each(|tile| {
                encode_tile(idx, tile);
                idx += 2;
            });

        self.board.ura_indicators.iter().copied().for_each(|tile| {
            encode_tile(idx, tile);
            idx += 2;
        });

        assert_eq!(idx, shape.0);
        arr
    }
}

#[rustfmt::skip]
const UNSHUFFLED: [Tile; 136] = [
    t!(1m),  t!(1m), t!(1m), t!(1m),
    t!(2m),  t!(2m), t!(2m), t!(2m),
    t!(3m),  t!(3m), t!(3m), t!(3m),
    t!(4m),  t!(4m), t!(4m), t!(4m),
    t!(5mr), t!(5m), t!(5m), t!(5m),
    t!(6m),  t!(6m), t!(6m), t!(6m),
    t!(7m),  t!(7m), t!(7m), t!(7m),
    t!(8m),  t!(8m), t!(8m), t!(8m),
    t!(9m),  t!(9m), t!(9m), t!(9m),

    t!(1p),  t!(1p), t!(1p), t!(1p),
    t!(2p),  t!(2p), t!(2p), t!(2p),
    t!(3p),  t!(3p), t!(3p), t!(3p),
    t!(4p),  t!(4p), t!(4p), t!(4p),
    t!(5pr), t!(5p), t!(5p), t!(5p),
    t!(6p),  t!(6p), t!(6p), t!(6p),
    t!(7p),  t!(7p), t!(7p), t!(7p),
    t!(8p),  t!(8p), t!(8p), t!(8p),
    t!(9p),  t!(9p), t!(9p), t!(9p),

    t!(1s),  t!(1s), t!(1s), t!(1s),
    t!(2s),  t!(2s), t!(2s), t!(2s),
    t!(3s),  t!(3s), t!(3s), t!(3s),
    t!(4s),  t!(4s), t!(4s), t!(4s),
    t!(5sr), t!(5s), t!(5s), t!(5s),
    t!(6s),  t!(6s), t!(6s), t!(6s),
    t!(7s),  t!(7s), t!(7s), t!(7s),
    t!(8s),  t!(8s), t!(8s), t!(8s),
    t!(9s),  t!(9s), t!(9s), t!(9s),

    t!(E), t!(E), t!(E), t!(E),
    t!(S), t!(S), t!(S), t!(S),
    t!(W), t!(W), t!(W), t!(W),
    t!(N), t!(N), t!(N), t!(N),
    t!(P), t!(P), t!(P), t!(P),
    t!(F), t!(F), t!(F), t!(F),
    t!(C), t!(C), t!(C), t!(C),
];
