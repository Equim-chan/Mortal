use super::player_list::{TENHOUI, TOP300_2K_GAMES};
use super::Grp;
use crate::arena::Board;
use crate::consts::ORACLE_OBS_SHAPE;
use crate::mjai::Event;
use crate::state::PlayerState;
use crate::tile::Tile;
use crate::{tu8, tuz};
use std::convert::identity;
use std::fs::File;
use std::io::prelude::*;
use std::iter;
use std::mem;

use anyhow::{bail, ensure, Context, Result};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use pyo3::PyObjectProtocol;
use rand::prelude::*;
use rayon::prelude::*;
use serde_json as json;

#[derive(Default)]
pub struct Invisible {
    // early -> late
    pub yama: Vec<Tile>,
    pub rinshan: Vec<Tile>,
    pub dora_indicators: Vec<Tile>,
    pub ura_indicators: Vec<Tile>,
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

impl Invisible {
    pub fn new(game: &[Event], trust_seed: bool) -> Vec<Self> {
        let mut ret = vec![];
        let mut cur = Self::default();
        let mut seed = None;
        let mut from_rinshan = false;
        let mut ura_is_recorded = false;
        let mut unknown_tiles = new_unknown_tiles();

        game.iter().for_each(|event| {
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
                        return;
                    } else {
                        cur.dora_indicators.push(*dora_marker);
                        unknown_tiles[dora_marker.as_usize()] -= 1;
                        tehais
                            .iter()
                            .flatten()
                            .for_each(|tile| unknown_tiles[tile.as_usize()] -= 1);
                    }
                }
                _ => (),
            };

            if seed.is_some() {
                return;
            }

            match event {
                Event::Tsumo { pai, .. } => {
                    if from_rinshan {
                        cur.rinshan.push(*pai);
                        from_rinshan = false;
                    } else {
                        cur.yama.push(*pai);
                        if cur.yama.len() > 70 {
                            panic!("yama size overflow");
                        }
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
                    ura.iter().for_each(|&tile| {
                        cur.ura_indicators.push(tile);
                        unknown_tiles[tile.as_usize()] -= 1;
                    });
                    ura_is_recorded = true;
                }
                Event::EndKyoku => {
                    let mut filler: Vec<_> = unknown_tiles
                        .into_iter()
                        .enumerate()
                        .filter(|&(_, count)| count > 0)
                        .flat_map(|(tid, count)| iter::repeat(Tile(tid as u8)).take(count as usize))
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
        });

        ret
    }

    // TODO: merge this this arena::board::BoardState::encode_oracle_obs; they
    // should be identical.
    fn encode(
        &self,
        opponent_states: &[PlayerState; 3],
        yama_idx: usize,
        rinshan_idx: usize,
    ) -> Array2<f32> {
        let mut arr = Array2::zeros(ORACLE_OBS_SHAPE);
        let mut idx = 0;

        opponent_states.iter().for_each(|state| {
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
            arr.slice_mut(s![idx..idx + n, ..]).fill(1.);
            idx += 6;

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

        self.yama[yama_idx..].iter().for_each(|&tile| {
            encode_tile(idx, tile);
            idx += 2;
        });
        // In real life case `self.yama[yama_idx..]` is at most 69 (`yama_idx`
        // is always >= 1), because the dealer always unconditionally deals the
        // first tile from yama. Therefore we do the minus one here.
        idx += (yama_idx - 1) * 2;

        self.rinshan[rinshan_idx..].iter().for_each(|&tile| {
            encode_tile(idx, tile);
            idx += 2;
        });
        idx += rinshan_idx * 2;

        self.dora_indicators.iter().for_each(|&tile| {
            encode_tile(idx, tile);
            idx += 2;
        });
        self.ura_indicators.iter().for_each(|&tile| {
            encode_tile(idx, tile);
            idx += 2;
        });

        assert_eq!(idx, ORACLE_OBS_SHAPE.0);
        arr
    }
}

#[pyclass]
#[pyo3(text_signature = "(
    *,
    oracle = True,
    player_name = None,
    trust_seed = True,
)")]
#[derive(Debug, Clone, Default)]
pub struct GameplayLoader {
    #[pyo3(get, set)]
    oracle: bool,
    #[pyo3(get, set)]
    player_name: Option<String>,
    #[pyo3(get, set)]
    trust_seed: bool,
}

#[pymethods]
impl GameplayLoader {
    #[new]
    #[args("*", oracle = "true", player_name = "None", trust_seed = "true")]
    const fn new(oracle: bool, player_name: Option<String>, trust_seed: bool) -> Self {
        Self {
            oracle,
            player_name,
            trust_seed,
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
}

#[pyproto]
impl PyObjectProtocol for GameplayLoader {
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

        let idxs = if let Some(player_name) = &self.player_name {
            match &events[0] {
                Event::StartGame { names, .. } => names
                    .iter()
                    .enumerate()
                    .filter(|(_, name)| name.as_str() == player_name)
                    .map(|(i, _)| i as u8)
                    .collect(),
                _ => bail!("the first event is not StartGame, got {:?}", events[0]),
            }
        } else {
            vec![0, 1, 2, 3]
        };

        idxs.into_par_iter()
            .map(|player_id| {
                Gameplay::load_events_by_player(self, events, player_id, invisibles.as_deref())
            })
            .collect()
    }
}

#[derive(Clone, Copy, derivative::Derivative)]
#[derivative(Default)]
#[repr(u8)]
pub enum Quality {
    LastPlace,
    #[derivative(Default)]
    Normal,
    Top300,
    Tenhoui,
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

    // one per kyoku
    pub grp: Grp,
    pub has_houjuu_while_not_waiting: Vec<bool>,

    // one per game
    pub player_id: u8,
    pub quality: Quality,
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
    fn take_grp(&mut self) -> Grp {
        mem::take(&mut self.grp)
    }
    // #[pyo3(text_signature = "($self, /)")]
    // fn take_shanten(&mut self) -> Vec<[i8; 4]> {
    //     mem::take(&mut self.shanten)
    // }

    #[pyo3(text_signature = "($self, /)")]
    fn take_apply_gamma(&mut self) -> Vec<bool> {
        mem::take(&mut self.apply_gamma)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_has_houjuu_while_not_waiting(&mut self) -> Vec<bool> {
        mem::take(&mut self.has_houjuu_while_not_waiting)
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_player_id(&mut self) -> u8 {
        self.player_id
    }
    #[pyo3(text_signature = "($self, /)")]
    fn take_quality(&mut self) -> u8 {
        self.quality as u8
    }
}

struct LoaderContext {
    state: PlayerState,
    kyoku_idx: usize,
    // fields below are only used for oracle
    opponent_states: [PlayerState; 3],
    from_rinshan: bool,
    yama_idx: usize,
    rinshan_idx: usize,
}

impl Gameplay {
    fn load_events_by_player(
        config: &GameplayLoader,
        events: &[Event],
        player_id: u8,
        invisibles: Option<&[Invisible]>,
    ) -> Result<Self> {
        let grp = Grp::load_events(events)?;
        let has_houjuu_while_not_waiting = vec![false; grp.len()];

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
            has_houjuu_while_not_waiting,
            player_id,
            quality,
            ..Default::default()
        };

        let mut ctx = LoaderContext {
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
        events
            .windows(4)
            .try_for_each(|wnd| data.extend_from_event_window(config, wnd, invisibles, &mut ctx))?;

        data.dones = data.at_kyoku.windows(2).map(|w| w[1] > w[0]).collect();
        data.dones.push(true);

        Ok(data)
    }

    fn extend_from_event_window(
        &mut self,
        config: &GameplayLoader,
        wnd: &[Event],
        invisibles: Option<&[Invisible]>,
        ctx: &mut LoaderContext,
    ) -> Result<()> {
        let cur = &wnd[0];
        let next = if matches!(wnd[1], Event::ReachAccepted { .. } | Event::Dora { .. }) {
            &wnd[2]
        } else {
            &wnd[1]
        };

        if matches!(cur, Event::EndKyoku) {
            ctx.kyoku_idx += 1;
        }

        if config.oracle {
            match cur {
                Event::EndKyoku => {
                    ctx.from_rinshan = false;
                    ctx.yama_idx = 0;
                    ctx.rinshan_idx = 0;
                }
                Event::Tsumo { .. } => {
                    if ctx.from_rinshan {
                        ctx.rinshan_idx += 1;
                        ctx.from_rinshan = false;
                    } else {
                        ctx.yama_idx += 1;
                    }
                }
                Event::Ankan { .. } | Event::Kakan { .. } | Event::Daiminkan { .. } => {
                    ctx.from_rinshan = true;
                }
                _ => (),
            };

            ctx.opponent_states.iter_mut().for_each(|s| {
                s.update(cur);
            });
        }

        let cans = ctx.state.update(cur);

        if !self.has_houjuu_while_not_waiting[ctx.kyoku_idx]
            && matches!(next, Event::Hora { actor, target, .. } if *actor != self.player_id && *target == self.player_id)
            && (ctx.state.shanten() > 0
                || ctx.state.at_furiten()
                || !ctx.state.waits().into_iter().any(identity))
        {
            self.has_houjuu_while_not_waiting[ctx.kyoku_idx] = true;
        };

        if !cans.can_act() {
            return Ok(());
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
            } if actor == self.player_id => {
                let high = consumed[0].deaka().as_u8().max(consumed[1].deaka().as_u8());
                let low = consumed[0].deaka().as_u8().min(consumed[1].deaka().as_u8());
                let tile_id = pai.deaka().as_u8();
                if tile_id < low {
                    Some(38)
                } else if tile_id > high {
                    Some(40)
                } else {
                    Some(39)
                }
            }
            Event::Pon { actor, .. } if actor == self.player_id => Some(41),
            Event::Daiminkan { actor, pai, .. } if actor == self.player_id => {
                kan_select = Some(pai.deaka().as_usize());
                Some(42)
            }
            Event::Kakan { pai, .. } => {
                // if state.kakan_candidates().len() > 1 {
                kan_select = Some(pai.deaka().as_usize());
                // }
                Some(42)
            }
            Event::Ankan { consumed, .. } => {
                // if state.ankan_candidates().len() > 1 {
                kan_select = Some(consumed[0].deaka().as_usize());
                // }
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
            let mut add_entry = |at_kan, label| {
                let (feature, mask) = ctx.state.encode_obs(at_kan);
                self.obs.push(feature);
                self.actions.push(label as i64);
                self.masks.push(mask);
                self.at_kyoku.push(ctx.kyoku_idx as u8);
                // only discard and kan will discount
                self.apply_gamma.push(label <= 37);
                if config.oracle {
                    // unwrap is safe because invisible is Some() iff
                    // `config.oracle` holds.
                    let invisible_obs = invisibles.unwrap()[ctx.kyoku_idx].encode(
                        &ctx.opponent_states,
                        ctx.yama_idx,
                        ctx.rinshan_idx,
                    );
                    self.invisible_obs.push(invisible_obs);
                }
            };

            add_entry(false, label);
            if let Some(kan) = kan_select {
                add_entry(true, kan);
            }
        }

        Ok(())
    }
}
