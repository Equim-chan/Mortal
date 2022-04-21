use super::BatchAgent;
use crate::consts::ACTION_SPACE;
use crate::mjai::{Event, EventExt, Metadata};
use crate::state::PlayerState;
use crate::tile::Tile;
use crate::tu8;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};
use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;

pub struct MortalBatchAgent {
    engine: PyObject,
    is_oracle: bool,
    enable_quick_eval: bool,
    name: String,
    player_ids: Vec<u8>,

    states: Vec<Array2<f32>>,
    invisible_states: Vec<Array2<f32>>,
    masks: Vec<Array1<bool>>,
    actions: Vec<usize>,

    q_values: Vec<[f32; ACTION_SPACE]>,
    masks_recv: Vec<[bool; ACTION_SPACE]>,
    is_greedy: Vec<bool>,
    last_eval_elapsed: Duration,
    last_batch_size: usize,

    evaluated: bool,
    action_idxs: Vec<usize>,
    kan_action_idxs: Vec<Option<usize>>,
    quick_eval_reactions: Vec<Option<Event>>,
}

impl MortalBatchAgent {
    pub fn new(engine: PyObject, player_ids: &[u8]) -> Result<Self> {
        let (name, is_oracle, enable_quick_eval) = Python::with_gil(|py| {
            let obj = engine.as_ref(py);
            ensure!(obj.getattr("react_batch")?.is_callable());

            let name = obj.getattr("name")?.extract()?;
            let is_oracle = obj.getattr("is_oracle")?.extract()?;
            let enable_quick_eval = obj.getattr("enable_quick_eval")?.extract()?;
            Ok((name, is_oracle, enable_quick_eval))
        })?;

        let size = player_ids.len();
        Ok(Self {
            engine,
            is_oracle,
            enable_quick_eval,
            name,
            player_ids: player_ids.to_vec(),

            states: vec![],
            invisible_states: vec![],
            masks: vec![],
            actions: vec![],

            q_values: vec![],
            masks_recv: vec![],
            is_greedy: vec![],
            last_eval_elapsed: Duration::ZERO,
            last_batch_size: 0,

            evaluated: false,
            action_idxs: vec![0; size],
            kan_action_idxs: vec![None; size],
            quick_eval_reactions: if enable_quick_eval {
                vec![None; size]
            } else {
                vec![]
            },
        })
    }

    fn evaluate(&mut self) -> Result<()> {
        if self.states.is_empty() {
            return Ok(());
        }

        let start = Instant::now();
        self.last_batch_size = self.states.len();

        (self.actions, self.q_values, self.masks_recv, self.is_greedy) = Python::with_gil(|py| {
            let states: Vec<_> = self
                .states
                .drain(..)
                .map(|v| PyArray2::from_owned_array(py, v))
                .collect();
            let masks: Vec<_> = self
                .masks
                .drain(..)
                .map(|v| PyArray1::from_owned_array(py, v))
                .collect();
            let invisible_states: Option<Vec<_>> = self.is_oracle.then(|| {
                self.invisible_states
                    .drain(..)
                    .map(|v| PyArray2::from_owned_array(py, v))
                    .collect()
            });

            let args = (states, masks, invisible_states);
            self.engine
                .as_ref(py)
                .call_method1("react_batch", args)
                .context("failed to execute `react_batch` on Python engine")?
                .extract()
                .context("failed to extract to Rust type")
        })?;

        self.last_eval_elapsed = Instant::now()
            .checked_duration_since(start)
            .unwrap_or(Duration::ZERO);

        Ok(())
    }

    fn gen_meta(&self, action_idx: usize) -> Metadata {
        let q_values = self.q_values[action_idx];
        let masks = self.masks_recv[action_idx];
        let is_greedy = self.is_greedy[action_idx];

        let mut mask_bits = 0;
        let q_values_compact = q_values
            .into_iter()
            .zip(masks)
            .enumerate()
            .filter(|(_, (_, m))| *m)
            .map(|(i, (q, _))| {
                mask_bits |= 0b1 << i;
                q
            })
            .collect();

        Metadata {
            q_values: Some(q_values_compact),
            mask_bits: Some(mask_bits),
            is_greedy: Some(is_greedy),
            ..Default::default()
        }
    }
}

impl BatchAgent for MortalBatchAgent {
    #[inline]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    fn need_oracle_obs(&self) -> bool {
        self.is_oracle
    }

    fn set_scene(
        &mut self,
        index: usize,
        _: &[EventExt],
        state: &PlayerState,
        invisible_state: Option<Array2<f32>>,
    ) -> Result<()> {
        self.evaluated = false;
        let cans = state.last_cans();

        if self.enable_quick_eval
            && cans.can_discard
            && !cans.can_riichi
            && !cans.can_tsumo_agari
            && !cans.can_ankan
            && !cans.can_kakan
            && !cans.can_ryukyoku
        {
            let candidates = state.discard_candidates_aka();
            let mut only_candidate = None;
            for (tile, &flag) in candidates.iter().enumerate() {
                if !flag {
                    continue;
                }
                if only_candidate.is_some() {
                    only_candidate = None;
                    break;
                }
                only_candidate = Some(tile);
            }

            if let Some(tile_id) = only_candidate {
                let actor = self.player_ids[index];
                let pai = Tile(tile_id as u8);
                let tsumogiri = state.last_self_tsumo().filter(|&t| t == pai).is_some();
                let ev = Event::Dahai {
                    actor,
                    pai,
                    tsumogiri,
                };
                self.quick_eval_reactions[index] = Some(ev);
                return Ok(());
            }
        }

        let need_kan_select = if cans.can_ankan || cans.can_kakan {
            let a = state.ankan_candidates();
            let k = state.kakan_candidates();
            a.len() + k.len() > 1 || !self.enable_quick_eval
        } else {
            false
        };

        if need_kan_select {
            let (kan_feature, kan_mask) = state.encode_obs(true);
            self.states.push(kan_feature);
            self.masks.push(kan_mask);
            if let Some(invisible_state) = invisible_state.clone() {
                self.invisible_states.push(invisible_state);
            }
            self.kan_action_idxs[index] = Some(self.states.len() - 1);
        }

        let (feature, mask) = state.encode_obs(false);
        self.states.push(feature);
        self.masks.push(mask);
        if let Some(invisible_state) = invisible_state {
            self.invisible_states.push(invisible_state);
        }
        self.action_idxs[index] = self.states.len() - 1;

        Ok(())
    }

    fn get_reaction(
        &mut self,
        index: usize,
        _: &[EventExt],
        state: &PlayerState,
        _: Option<Array2<f32>>,
    ) -> Result<EventExt> {
        if self.enable_quick_eval {
            if let Some(ev) = self.quick_eval_reactions[index].take() {
                return Ok(EventExt::no_meta(ev));
            }
        }

        if !self.evaluated {
            self.evaluate()?;
            self.evaluated = true;
        }
        let start = Instant::now();

        let action_idx = self.action_idxs[index];
        let kan_select_idx = self.kan_action_idxs[index].take();

        let actor = self.player_ids[index];
        let akas_in_hand = state.akas_in_hand();
        let cans = state.last_cans();

        let action = self.actions[action_idx];
        let event = match action {
            0..=36 => {
                ensure!(
                    cans.can_discard,
                    "failed discard check: {}",
                    state.brief_info()
                );

                let pai = Tile(action as u8);
                let tsumogiri = state.last_self_tsumo().filter(|&t| t == pai).is_some();
                Event::Dahai {
                    actor,
                    pai,
                    tsumogiri,
                }
            }

            37 => {
                ensure!(
                    cans.can_riichi,
                    "failed riichi check: {}",
                    state.brief_info()
                );

                Event::Reach { actor }
            }

            38 => {
                ensure!(
                    cans.can_chi_low,
                    "failed chi low check: {}",
                    state.brief_info()
                );

                let pai = state
                    .last_kawa_tile()
                    .context("invalid state: no last kawa tile")?;
                let tile_id = pai.deaka().as_u8();

                let can_akaize_consumed = match pai.as_u8() {
                    tu8!(3m) | tu8!(4m) => akas_in_hand[0],
                    tu8!(3p) | tu8!(4p) => akas_in_hand[1],
                    tu8!(3s) | tu8!(4s) => akas_in_hand[2],
                    _ => false,
                };
                let consumed = if can_akaize_consumed {
                    [Tile(tile_id + 1).akaize(), Tile(tile_id + 2).akaize()]
                } else {
                    [Tile(tile_id + 1), Tile(tile_id + 2)]
                };
                Event::Chi {
                    actor,
                    target: cans.target_actor,
                    pai,
                    consumed,
                }
            }
            39 => {
                ensure!(
                    cans.can_chi_mid,
                    "failed chi mid check: {}",
                    state.brief_info()
                );

                let pai = state
                    .last_kawa_tile()
                    .context("invalid state: no last kawa tile")?;
                let tile_id = pai.deaka().as_u8();

                let can_akaize_consumed = match pai.as_u8() {
                    tu8!(4m) | tu8!(6m) => akas_in_hand[0],
                    tu8!(4p) | tu8!(6p) => akas_in_hand[1],
                    tu8!(4s) | tu8!(6s) => akas_in_hand[2],
                    _ => false,
                };
                let consumed = if can_akaize_consumed {
                    [Tile(tile_id - 1).akaize(), Tile(tile_id + 1).akaize()]
                } else {
                    [Tile(tile_id - 1), Tile(tile_id + 1)]
                };
                Event::Chi {
                    actor,
                    target: cans.target_actor,
                    pai,
                    consumed,
                }
            }
            40 => {
                ensure!(
                    cans.can_chi_high,
                    "failed chi high check: {}",
                    state.brief_info()
                );

                let pai = state
                    .last_kawa_tile()
                    .context("invalid state: no last kawa tile")?;
                let tile_id = pai.deaka().as_u8();

                let can_akaize_consumed = match pai.as_u8() {
                    tu8!(6m) | tu8!(7m) => akas_in_hand[0],
                    tu8!(6p) | tu8!(7p) => akas_in_hand[1],
                    tu8!(6s) | tu8!(7s) => akas_in_hand[2],
                    _ => false,
                };
                let consumed = if can_akaize_consumed {
                    [Tile(tile_id - 2).akaize(), Tile(tile_id - 1).akaize()]
                } else {
                    [Tile(tile_id - 2), Tile(tile_id - 1)]
                };
                Event::Chi {
                    actor,
                    target: cans.target_actor,
                    pai,
                    consumed,
                }
            }

            41 => {
                ensure!(cans.can_pon, "failed pon check: {}", state.brief_info());

                let pai = state
                    .last_kawa_tile()
                    .context("invalid state: no last kawa tile")?;

                let can_akaize_consumed = match pai.as_u8() {
                    tu8!(5m) => akas_in_hand[0],
                    tu8!(5p) => akas_in_hand[1],
                    tu8!(5s) => akas_in_hand[2],
                    _ => false,
                };
                let consumed = if can_akaize_consumed {
                    [pai.akaize(), pai.deaka()]
                } else {
                    [pai.deaka(); 2]
                };
                Event::Pon {
                    actor,
                    target: cans.target_actor,
                    pai,
                    consumed,
                }
            }

            42 => {
                ensure!(
                    cans.can_daiminkan || cans.can_ankan || cans.can_kakan,
                    "failed kan check: {}",
                    state.brief_info()
                );

                let ankan_candidates = state.ankan_candidates();
                let kakan_candidates = state.kakan_candidates();

                let tile = if let Some(kan_idx) = kan_select_idx {
                    let tid = self.actions[kan_idx] as u8;
                    ensure!(
                        ankan_candidates.contains(&tid) || kakan_candidates.contains(&tid),
                        "kan choice not in kan candidates: {}",
                        state.brief_info()
                    );
                    Tile(tid)
                } else if cans.can_daiminkan {
                    state
                        .last_kawa_tile()
                        .context("invalid state: no last kawa tile")?
                } else if cans.can_ankan {
                    let tid = ankan_candidates[0];
                    Tile(tid)
                } else {
                    let tid = kakan_candidates[0];
                    Tile(tid)
                };

                if cans.can_daiminkan {
                    let consumed = if tile.is_aka() {
                        [tile.deaka(); 3]
                    } else {
                        [tile.akaize(), tile, tile]
                    };
                    Event::Daiminkan {
                        actor,
                        target: cans.target_actor,
                        pai: tile,
                        consumed,
                    }
                } else if cans.can_ankan && ankan_candidates.contains(&tile.deaka().as_u8()) {
                    Event::Ankan {
                        actor,
                        consumed: [tile.akaize(), tile, tile, tile],
                    }
                } else {
                    let can_akaize_target = match tile.as_u8() {
                        tu8!(5m) => akas_in_hand[0],
                        tu8!(5p) => akas_in_hand[1],
                        tu8!(5s) => akas_in_hand[2],
                        _ => false,
                    };
                    let (pai, consumed) = if can_akaize_target {
                        (tile.akaize(), [tile.deaka(); 3])
                    } else {
                        (tile.deaka(), [tile.akaize(), tile.deaka(), tile.deaka()])
                    };
                    Event::Kakan {
                        actor,
                        pai,
                        consumed,
                    }
                }
            }

            43 => {
                ensure!(
                    cans.can_tsumo_agari || cans.can_ron_agari,
                    "failed hora check: {}",
                    state.brief_info()
                );

                Event::Hora {
                    actor,
                    target: cans.target_actor,
                    deltas: None,
                    ura_markers: None,
                }
            }

            44 => {
                ensure!(
                    cans.can_ryukyoku,
                    "failed ryukyoku check: {}",
                    state.brief_info()
                );

                Event::Ryukyoku { deltas: None }
            }

            // 45
            _ => Event::None,
        };

        let mut meta = self.gen_meta(action_idx);
        let eval_time_ns = Instant::now()
            .checked_duration_since(start)
            .unwrap_or(Duration::ZERO)
            .saturating_add(self.last_eval_elapsed)
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX);

        meta.eval_time_ns = Some(eval_time_ns);
        meta.batch_size = Some(self.last_batch_size);
        meta.kan_select = kan_select_idx.map(|kan_idx| Box::new(self.gen_meta(kan_idx)));

        Ok(EventExt {
            event,
            meta: Some(meta),
        })
    }
}
