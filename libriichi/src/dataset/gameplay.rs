use super::invisible::Invisible;
use super::player_list::{TENHOUI, TOP300_2K_GAMES};
use super::Grp;
use crate::chi_type::ChiType;
use crate::mjai::Event;
use crate::state::PlayerState;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

use anyhow::{bail, ensure, Context, Result};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use rayon::prelude::*;
use serde_json as json;
use tinyvec::ArrayVec;

#[pyclass]
#[pyo3(text_signature = "(
    *,
    oracle = True,
    player_name = None,
    excludes = None,
    trust_seed = False,
    always_include_kyoku_select = True,
)")]
#[derive(Debug, Clone, Default)]
pub struct GameplayLoader {
    #[pyo3(get, set)]
    oracle: bool,
    #[pyo3(get, set)]
    player_name: Option<String>,
    #[pyo3(get, set)]
    excludes: Option<Vec<String>>,
    #[pyo3(get, set)]
    trust_seed: bool,
    #[pyo3(get, set)]
    always_include_kyoku_select: bool,
}

#[pyclass]
#[derive(Clone, Default)]
pub struct Gameplay {
    // one per move
    pub obs: Vec<Array2<f32>>,
    pub invisible_obs: Vec<Array2<f32>>,
    pub actions: Vec<i64>,
    pub masks: Vec<Array1<bool>>,
    pub at_kyoku: Vec<u8>,
    pub dones: Vec<bool>,
    pub apply_gamma: Vec<bool>,
    pub at_turns: Vec<u8>,
    pub shantens: Vec<i8>,

    // one per kyoku
    pub grp: Grp,

    // one per game
    pub player_id: u8,
    pub player_name: String,
    pub quality: Quality,
}

#[pyclass]
#[derive(Clone, Copy, Default)]
pub enum Quality {
    LastPlace,
    #[default]
    Normal,
    Top300,
    Tenhoui,
}

struct LoaderContext<'a> {
    config: &'a GameplayLoader,
    invisibles: Option<&'a [Invisible]>,

    state: PlayerState,
    kyoku_idx: usize,
    // fields below are only used for oracle
    opponent_states: [PlayerState; 3],
    from_rinshan: bool,
    yama_idx: usize,
    rinshan_idx: usize,
}

#[pymethods]
impl GameplayLoader {
    #[new]
    #[args(
        "*",
        oracle = "true",
        player_name = "None",
        excludes = "None",
        trust_seed = "false",
        always_include_kyoku_select = "true"
    )]
    const fn new(
        oracle: bool,
        player_name: Option<String>,
        excludes: Option<Vec<String>>,
        trust_seed: bool,
        always_include_kyoku_select: bool,
    ) -> Self {
        Self {
            oracle,
            player_name,
            excludes,
            trust_seed,
            always_include_kyoku_select,
        }
    }

    // Nested result is too hard to handle...
    #[pyo3(text_signature = "($self, raw_log, /)")]
    fn load_log(&self, raw_log: &str) -> Result<Vec<Gameplay>> {
        let events = raw_log
            .lines()
            .map(json::from_str)
            .collect::<Result<Vec<Event>, _>>()
            .context("failed to parse log")?;
        self.load_events(&events)
    }

    #[pyo3(name = "load_gz_log_files")]
    #[pyo3(text_signature = "($self, gzip_filenames, /)")]
    fn load_gz_log_files_py(&self, gzip_filenames: Vec<String>) -> Result<Vec<Gameplay>> {
        self.load_gz_log_files(gzip_filenames)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl GameplayLoader {
    pub fn load_gz_log_files<V, S>(&self, gzip_filenames: V) -> Result<Vec<Gameplay>>
    where
        V: IntoParallelIterator<Item = S>,
        S: AsRef<str>,
    {
        let res: Result<Vec<Vec<_>>> = gzip_filenames
            .into_par_iter()
            .map(|f| {
                let filename = f.as_ref();
                let inner = || {
                    let file = File::open(filename)?;
                    let mut gz = GzDecoder::new(file);
                    let mut raw = String::new();
                    gz.read_to_string(&mut raw)?;
                    self.load_log(&raw)
                };
                inner().with_context(|| format!("error when reading {filename}"))
            })
            .collect();
        Ok(res?.into_iter().flatten().collect())
    }

    pub fn load_events(&self, events: &[Event]) -> Result<Vec<Gameplay>> {
        let invisibles = self.oracle.then(|| Invisible::new(events, self.trust_seed));

        let idxs: ArrayVec<[u8; 4]> = match &events[0] {
            Event::StartGame { names, .. } => names
                .iter()
                .enumerate()
                .filter(|(_, name)| {
                    if let Some(player_name) = &self.player_name {
                        return player_name == name.as_str();
                    }
                    if let Some(ex) = &self.excludes {
                        if ex.contains(name) {
                            return false;
                        }
                    }
                    true
                })
                .map(|(i, _)| i as u8)
                .collect(),
            _ => bail!("the first event is not StartGame, got {:?}", events[0]),
        };

        idxs.into_par_iter()
            .map(|&player_id| {
                Gameplay::load_events_by_player(self, events, player_id, invisibles.as_deref())
            })
            .collect()
    }
}

#[pymethods]
impl Gameplay {
    #[pyo3(text_signature = "($self, /)")]
    fn take_obs<'py>(&mut self, py: Python<'py>) -> Vec<&'py PyArray2<f32>> {
        mem::take(&mut self.obs)
            .into_iter()
            .map(|v| PyArray2::from_owned_array(py, v))
            .collect()
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_invisible_obs<'py>(&mut self, py: Python<'py>) -> Vec<&'py PyArray2<f32>> {
        mem::take(&mut self.invisible_obs)
            .into_iter()
            .map(|v| PyArray2::from_owned_array(py, v))
            .collect()
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_actions(&mut self) -> Vec<i64> {
        mem::take(&mut self.actions)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_masks<'py>(&mut self, py: Python<'py>) -> Vec<&'py PyArray1<bool>> {
        mem::take(&mut self.masks)
            .into_iter()
            .map(|v| PyArray1::from_owned_array(py, v))
            .collect()
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_at_kyoku(&mut self) -> Vec<u8> {
        mem::take(&mut self.at_kyoku)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_dones(&mut self) -> Vec<bool> {
        mem::take(&mut self.dones)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_apply_gamma(&mut self) -> Vec<bool> {
        mem::take(&mut self.apply_gamma)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_at_turns(&mut self) -> Vec<u8> {
        mem::take(&mut self.at_turns)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_shantens(&mut self) -> Vec<i8> {
        mem::take(&mut self.shantens)
    }

    #[pyo3(text_signature = "($self, /)")]
    fn take_grp(&mut self) -> Grp {
        mem::take(&mut self.grp)
    }

    #[pyo3(text_signature = "($self, /)")]
    const fn take_player_id(&self) -> u8 {
        self.player_id
    }
    #[pyo3(text_signature = "($self, /)")]
    const fn take_quality(&self) -> Quality {
        self.quality
    }
}

impl Gameplay {
    fn load_events_by_player(
        config: &GameplayLoader,
        events: &[Event],
        player_id: u8,
        invisibles: Option<&[Invisible]>,
    ) -> Result<Self> {
        let grp = Grp::load_events(events)?;

        let quality = if let Some(Event::StartGame { names, .. }) = events.get(0) {
            let name = names[player_id as usize].as_str();
            ensure!(!name.is_empty(), "player name is empty");
            if TENHOUI.contains(&name) {
                Quality::Tenhoui
            } else if TOP300_2K_GAMES.get_key_id(name).is_some() {
                Quality::Top300
            } else if grp.rank_by_player[player_id as usize] == 3 {
                Quality::LastPlace
            } else {
                Quality::Normal
            }
        } else {
            bail!("first event is not start_game");
        };

        let mut data = Self {
            grp,
            player_id,
            quality,
            ..Default::default()
        };

        let mut ctx = LoaderContext {
            config,
            invisibles,
            state: PlayerState::new(player_id),
            kyoku_idx: 0,
            opponent_states: [
                PlayerState::new((player_id + 1) % 4),
                PlayerState::new((player_id + 2) % 4),
                PlayerState::new((player_id + 3) % 4),
            ],
            from_rinshan: false,
            yama_idx: 0,
            rinshan_idx: 0,
        };

        // It is guaranteed that there are at least 4 events.
        // tsumo/dahai -> ryukyoku/hora -> end kyoku -> end game
        for wnd in events.windows(4) {
            data.extend_from_event_window(&mut ctx, wnd.try_into().unwrap());
        }

        data.dones = data.at_kyoku.windows(2).map(|w| w[1] > w[0]).collect();
        data.dones.push(true);

        Ok(data)
    }

    // Inlined because its callsite is extremely hot.
    #[inline(always)]
    fn extend_from_event_window(&mut self, ctx: &mut LoaderContext<'_>, wnd: &[Event; 4]) {
        let LoaderContext {
            config,
            invisibles,
            state,
            kyoku_idx,
            opponent_states,
            from_rinshan,
            yama_idx,
            rinshan_idx,
        } = ctx;

        let cur = &wnd[0];
        let next = if matches!(wnd[1], Event::ReachAccepted { .. } | Event::Dora { .. }) {
            &wnd[2]
        } else {
            &wnd[1]
        };

        match cur {
            Event::StartGame { names, .. } => {
                self.player_name = names[self.player_id as usize].clone();
            }
            Event::EndKyoku => {
                *kyoku_idx += 1;
            }
            _ => (),
        }

        if invisibles.is_some() {
            match cur {
                Event::EndKyoku => {
                    *from_rinshan = false;
                    *yama_idx = 0;
                    *rinshan_idx = 0;
                }
                Event::Tsumo { .. } => {
                    if *from_rinshan {
                        *rinshan_idx += 1;
                        *from_rinshan = false;
                    } else {
                        *yama_idx += 1;
                    }
                }
                Event::Ankan { .. } | Event::Kakan { .. } | Event::Daiminkan { .. } => {
                    *from_rinshan = true;
                }
                _ => (),
            };

            for s in opponent_states {
                s.update(cur);
            }
        }

        let cans = state.update(cur);
        if !cans.can_act() {
            return;
        }

        let mut kan_select = None;
        let label_opt = match *next {
            Event::Dahai { pai, .. } => Some(pai.as_usize()),
            Event::Reach { .. } => Some(37),
            Event::Chi {
                actor,
                pai,
                consumed,
                ..
            } if actor == self.player_id => match ChiType::new(consumed, pai) {
                ChiType::Low => Some(38),
                ChiType::Mid => Some(39),
                ChiType::High => Some(40),
            },
            Event::Pon { actor, .. } if actor == self.player_id => Some(41),
            Event::Daiminkan { actor, pai, .. } if actor == self.player_id => {
                if config.always_include_kyoku_select {
                    kan_select = Some(pai.deaka().as_usize());
                }
                Some(42)
            }
            Event::Kakan { pai, .. } => {
                if config.always_include_kyoku_select || state.kakan_candidates().len() > 1 {
                    kan_select = Some(pai.deaka().as_usize());
                }
                Some(42)
            }
            Event::Ankan { consumed, .. } => {
                if config.always_include_kyoku_select || state.ankan_candidates().len() > 1 {
                    kan_select = Some(consumed[0].deaka().as_usize());
                }
                Some(42)
            }
            Event::Ryukyoku { .. } if cans.can_ryukyoku => Some(44),
            _ => {
                let mut ret = None;

                let mut has_any_ron = false;
                if let Event::Hora { .. } = &wnd[1] {
                    has_any_ron = true;
                    // Check if the POV is one of those who made Hora.
                    for ev in &wnd[1..] {
                        match *ev {
                            Event::EndKyoku { .. } => break,
                            Event::Hora { actor, .. } if actor == self.player_id => {
                                ret = Some(43);
                                break;
                            }
                            _ => (),
                        };
                    }
                }

                if ret.is_none() {
                    // It is now proven there is no ron from the POV.
                    if cans.can_chi() && matches!(next, Event::Tsumo { .. })
                        || (cans.can_pon || cans.can_daiminkan || cans.can_ron_agari)
                            && !has_any_ron
                    {
                        // Can chi, but actively denied instead of being
                        // interrupted by other's pon/daiminkan/ron.
                        //
                        // or
                        //
                        // Can pon/daiminkan/ron, but actively denied
                        // instead of being interrupted by other's ron.
                        ret = Some(45);
                    }
                }

                ret
            }
        };

        if let Some(label) = label_opt {
            self.add_entry(ctx, false, label);
            if let Some(kan) = kan_select {
                self.add_entry(ctx, true, kan);
            }
        }
    }

    fn add_entry(&mut self, ctx: &LoaderContext<'_>, at_kan_select: bool, label: usize) {
        let (feature, mask) = ctx.state.encode_obs(at_kan_select);
        self.obs.push(feature);
        self.actions.push(label as i64);
        self.masks.push(mask);
        self.at_kyoku.push(ctx.kyoku_idx as u8);
        // only discard and kan will discount
        self.apply_gamma.push(label <= 37);
        self.at_turns.push(ctx.state.at_turn());
        self.shantens.push(ctx.state.shanten());

        if let Some(invisibles) = ctx.invisibles {
            let invisible_obs = invisibles[ctx.kyoku_idx].encode(
                &ctx.opponent_states,
                ctx.yama_idx,
                ctx.rinshan_idx,
            );
            self.invisible_obs.push(invisible_obs);
        }
    }
}
