use crate::consts::GRP_SIZE;
use crate::mjai::Event;
use crate::tu8;
use crate::vec_ops::vec_add_assign;
use std::array;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use ndarray::prelude::*;
use numpy::PyArray2;
use pyo3::prelude::*;
use rayon::prelude::*;
use serde_json as json;
use tinyvec::array_vec;

#[pyclass]
#[derive(Clone, Default)]
pub struct Grp {
    // [grand_kyoku, honba, kyotaku, [score[i] / 10000]] where i is player_id
    pub feature: Array2<f64>,
    pub rank_by_player: [u8; 4],
    pub final_scores: [i32; 4],
}

#[pymethods]
impl Grp {
    #[staticmethod]
    #[pyo3(text_signature = "(raw_log, /)")]
    fn load_log(raw_log: &str) -> Result<Self> {
        let events = raw_log
            .lines()
            .map(json::from_str)
            .collect::<Result<Vec<Event>, _>>()
            .context("failed to parse log")?;
        Self::load_events(&events)
    }

    #[staticmethod]
    #[pyo3(name = "load_gz_log_files")]
    #[pyo3(text_signature = "(gzip_filenames, /)")]
    fn load_gz_log_files_py(gzip_filenames: Vec<&str>) -> Result<Vec<Self>> {
        Self::load_gz_log_files(gzip_filenames)
    }

    /// Returns List[List[np.ndarray]]
    #[pyo3(text_signature = "($self, /)")]
    pub fn take_feature<'py>(&mut self, py: Python<'py>) -> &'py PyArray2<f64> {
        PyArray2::from_owned_array(py, mem::take(&mut self.feature))
    }
    #[pyo3(text_signature = "($self, /)")]
    pub const fn take_rank_by_player(&self) -> [u8; 4] {
        self.rank_by_player
    }
    #[pyo3(text_signature = "($self, /)")]
    pub const fn take_final_scores(&self) -> [i32; 4] {
        self.final_scores
    }
}

impl Grp {
    #[inline]
    pub fn len(&self) -> usize {
        self.feature.len_of(Axis(0))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn load_gz_log_files<V, S>(gzip_filenames: V) -> Result<Vec<Self>>
    where
        V: IntoParallelIterator<Item = S>,
        S: AsRef<str>,
    {
        gzip_filenames
            .into_par_iter()
            .map(|f| {
                let filename = f.as_ref();
                let inner = || {
                    let file = File::open(filename)?;
                    let mut gz = GzDecoder::new(file);
                    let mut raw = String::new();
                    gz.read_to_string(&mut raw)?;
                    Self::load_log(&raw)
                };
                inner().with_context(|| format!("error when reading {filename}"))
            })
            .collect()
    }

    pub fn load_events(events: &[Event]) -> Result<Self> {
        let mut game_info = vec![];
        let mut rank_by_player_opt = None;
        let mut final_deltas = [0; 4];
        let mut final_scores = [0; 4];
        let mut cur_kyotaku = 0;

        for ev in events.iter().rev() {
            match *ev {
                Event::Hora { deltas, .. } | Event::Ryukyoku { deltas, .. } => {
                    if rank_by_player_opt.is_none() {
                        let ds = deltas.context(
                            "invalid log: field `deltas` is required for Hora and Ryukyoku of AL",
                        )?;
                        vec_add_assign(&mut final_deltas, &ds);
                    }
                }
                Event::ReachAccepted { actor } => {
                    if rank_by_player_opt.is_none() {
                        final_deltas[actor as usize] -= 1000;
                        cur_kyotaku += 1;
                    }
                }
                Event::StartKyoku {
                    bakaze,
                    kyoku,
                    honba,
                    kyotaku,
                    scores,
                    ..
                } => {
                    if rank_by_player_opt.is_none() {
                        final_scores = scores;
                        vec_add_assign(&mut final_scores, &final_deltas);

                        // (player_id, score)
                        let mut player_by_rank: [_; 4] = array::from_fn(|i| (i, final_scores[i]));
                        player_by_rank.sort_by_key(|(_, s)| -s);

                        // assume the sum of scores should be 100k
                        let sum: i32 = final_scores.iter().sum();
                        if sum < 100_000 {
                            final_scores[player_by_rank[0].0] +=
                                (kyotaku as i32 + cur_kyotaku) * 1000;
                        }

                        let mut rank_by_player = [0; 4];
                        for (rank, (player_id, _)) in player_by_rank.into_iter().enumerate() {
                            rank_by_player[player_id] = rank as u8;
                        }
                        rank_by_player_opt = Some(rank_by_player);
                    }

                    let mut kyoku_info = array_vec!([_; GRP_SIZE]);
                    let grand_kyoku = match bakaze.as_u8() {
                        tu8!(E) => kyoku - 1,
                        tu8!(S) => 3 + kyoku,
                        _ => 7 + kyoku,
                    };
                    kyoku_info.push(grand_kyoku as f64);
                    kyoku_info.push(honba as f64);
                    kyoku_info.push(kyotaku as f64);
                    // assume player 0 is the oya at E1
                    kyoku_info.extend(scores.iter().map(|&score| score as f64 / 10000.));
                    assert_eq!(kyoku_info.len(), GRP_SIZE);

                    game_info.insert(0, kyoku_info);
                }
                Event::EndKyoku if rank_by_player_opt.is_none() => cur_kyotaku = 0,
                _ => (),
            }
        }

        let rank_by_player =
            rank_by_player_opt.context("invalid log: no Hora or Ryukyoku after a StartKyoku")?;
        let shape = (game_info.len(), GRP_SIZE);
        let feature = Array::from_iter(game_info.into_iter().flatten()).into_shape(shape)?;

        Ok(Self {
            feature,
            rank_by_player,
            final_scores,
        })
    }
}
