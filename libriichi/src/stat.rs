use crate::algo::point::Point;
use crate::mjai::Event;
use crate::py_helper::add_submodule;
use crate::rankings::Rankings;
use crate::vec_ops::vec_add_assign;
use std::fmt;
use std::fs::File;
use std::io;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use derive_more::{Add, AddAssign, Sum};
use flate2::read::GzDecoder;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use rayon::prelude::*;
use serde_json as json;

/// Notes:
///
/// - All the Δscore about riichi do not cover the 1000 kyotaku of its
///   sengenhai, but do cover all other kyotakus.
/// - Deal-in After Riichi is recognized at the moment the sengenhai is
///   discarded.
/// - Every other Δscore cover kyotakus.
/// - Ankan is not recognized as fuuro.
#[pyclass]
#[derive(Debug, Clone, Default, PartialEq, Eq, Add, AddAssign, Sum)]
pub struct Stat {
    #[pyo3(get, set)]
    pub game: i64,
    #[pyo3(get, set)]
    pub round: i64,
    #[pyo3(get, set)]
    pub oya: i64,

    #[pyo3(get, set)]
    pub point: i64,
    #[pyo3(get, set)]
    pub rank_1: i64,
    #[pyo3(get, set)]
    pub rank_2: i64,
    #[pyo3(get, set)]
    pub rank_3: i64,
    #[pyo3(get, set)]
    pub rank_4: i64,
    #[pyo3(get, set)]
    pub tobi: i64,

    #[pyo3(get, set)]
    pub fuuro: i64,
    #[pyo3(get, set)]
    pub fuuro_num: i64,
    #[pyo3(get, set)]
    pub fuuro_point: i64,
    #[pyo3(get, set)]
    pub fuuro_agari: i64,
    #[pyo3(get, set)]
    pub fuuro_agari_jun: i64,
    #[pyo3(get, set)]
    pub fuuro_agari_point: i64,
    #[pyo3(get, set)]
    pub fuuro_houjuu: i64,
    #[pyo3(get, set)]
    pub agari: i64,
    #[pyo3(get, set)]
    pub agari_as_oya: i64,
    #[pyo3(get, set)]
    pub agari_jun: i64,
    #[pyo3(get, set)]
    pub agari_point_oya: i64,
    #[pyo3(get, set)]
    pub agari_point_ko: i64,

    #[pyo3(get, set)]
    pub houjuu: i64,
    #[pyo3(get, set)]
    pub houjuu_jun: i64,
    #[pyo3(get, set)]
    pub houjuu_to_oya: i64,
    #[pyo3(get, set)]
    pub houjuu_point_to_oya: i64,
    #[pyo3(get, set)]
    pub houjuu_point_to_ko: i64,

    #[pyo3(get, set)]
    pub riichi: i64,
    #[pyo3(get, set)]
    pub riichi_as_oya: i64,
    #[pyo3(get, set)]
    pub riichi_jun: i64,
    #[pyo3(get, set)]
    pub riichi_agari: i64,
    #[pyo3(get, set)]
    pub riichi_agari_point: i64,
    #[pyo3(get, set)]
    pub riichi_agari_jun: i64,
    #[pyo3(get, set)]
    pub riichi_houjuu: i64,
    #[pyo3(get, set)]
    pub riichi_ryukyoku: i64,
    #[pyo3(get, set)]
    pub riichi_point: i64,
    #[pyo3(get, set)]
    pub chasing_riichi: i64,
    #[pyo3(get, set)]
    pub riichi_got_chased: i64,

    #[pyo3(get, set)]
    pub dama_agari: i64,
    #[pyo3(get, set)]
    pub dama_agari_jun: i64,
    #[pyo3(get, set)]
    pub dama_agari_point: i64,

    #[pyo3(get, set)]
    pub ryukyoku: i64,
    #[pyo3(get, set)]
    pub ryukyoku_point: i64,

    #[pyo3(get, set)]
    pub yakuman: i64,
    #[pyo3(get, set)]
    pub nagashi_mangan: i64,
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Games            {}
Rounds           {}
Rounds as dealer {}

1st (rate)       {} ({:.6})
2nd (rate)       {} ({:.6})
3rd (rate)       {} ({:.6})
4th (rate)       {} ({:.6})
Tobi(rate)       {} ({:.6})
Avg rank         {:.6}
Total rank pt    {}
Avg rank pt      {:.6}
Total Δscore     {}
Avg game Δscore  {:.6}
Avg round Δscore {:.6}

Win rate      {:.6}
Deal-in rate  {:.6}
Call rate     {:.6}
Riichi rate   {:.6}
Ryukyoku rate {:.6}

Avg winning Δscore               {:.6}
Avg winning Δscore as dealer     {:.6}
Avg winning Δscore as non-dealer {:.6}
Avg riichi winning Δscore        {:.6}
Avg open winning Δscore          {:.6}
Avg dama winning Δscore          {:.6}
Avg ryukyoku Δscore              {:.6}

Avg winning turn        {:.6}
Avg riichi winning turn {:.6}
Avg open winning turn   {:.6}
Avg dama winning turn   {:.6}

Avg deal-in turn                 {:.6}
Avg deal-in Δscore               {:.6}
Avg deal-in Δscore to dealer     {:.6}
Avg deal-in Δscore to non-dealer {:.6}

Chasing riichi rate       {:.6}
Riichi chased rate        {:.6}
Winning rate after riichi {:.6}
Deal-in rate after riichi {:.6}
Avg riichi turn           {:.6}
Avg riichi Δscore         {:.6}

Avg number of calls     {:.6}
Winning rate after call {:.6}
Deal-in rate after call {:.6}
Avg call Δscore         {:.6}

Dealer wins/all dealer rounds  {:.6}
Dealer wins/all wins           {:.6}
Deal-in to dealer/all deal-ins {:.6}

Yakuman (rate)        {} ({:.9})
Nagashi mangan (rate) {} ({:.9})"#,
            self.game,
            self.round,
            self.oya,
            //
            self.rank_1,
            self.rank_1_rate(),
            self.rank_2,
            self.rank_2_rate(),
            self.rank_3,
            self.rank_3_rate(),
            self.rank_4,
            self.rank_4_rate(),
            self.tobi,
            self.tobi_rate(),
            self.avg_rank(),
            self.total_pt([90, 45, 0, -135]),
            self.avg_pt([90, 45, 0, -135]),
            self.point,
            self.avg_point_per_game(),
            self.avg_point_per_round(),
            //
            self.agari_rate(),
            self.houjuu_rate(),
            self.fuuro_rate(),
            self.riichi_rate(),
            self.ryukyoku_rate(),
            //
            self.avg_point_per_agari(),
            self.avg_point_per_oya_agari(),
            self.avg_point_per_ko_agari(),
            self.avg_point_per_riichi_agari(),
            self.avg_point_per_fuuro_agari(),
            self.avg_point_per_dama_agari(),
            self.avg_point_per_ryukyoku(),
            //
            self.avg_agari_jun(),
            self.avg_riichi_agari_jun(),
            self.avg_fuuro_agari_jun(),
            self.avg_dama_agari_jun(),
            //
            self.avg_houjuu_jun(),
            self.avg_point_per_houjuu(),
            self.avg_point_per_houjuu_to_oya(),
            self.avg_point_per_houjuu_to_ko(),
            //
            self.chasing_riichi_rate(),
            self.riichi_chased_rate(),
            self.agari_rate_after_riichi(),
            self.houjuu_rate_after_riichi(),
            self.avg_riichi_jun(),
            self.avg_riichi_point(),
            //
            self.avg_fuuro_num(),
            self.agari_rate_after_fuuro(),
            self.houjuu_rate_after_fuuro(),
            self.avg_fuuro_point(),
            //
            self.agari_rate_as_oya(),
            self.agari_as_oya_rate(),
            self.houjuu_to_oya_rate(),
            //
            self.yakuman,
            self.yakuman_rate(),
            self.nagashi_mangan,
            self.nagashi_mangan_rate(),
        )
    }
}

impl Stat {
    /// We do not use `add_game(&mut self)` here as `Stat` impls `Add` and `Sum` so we
    /// can use rayon easier.
    #[must_use]
    pub fn from_game(events: &[Event], player_id: u8) -> Self {
        let mut stat = Self {
            game: 1,
            ..Default::default()
        };

        let mut cur_scores = [0; 4];
        let mut riichi_declared = false;
        let mut riichi_accepted = false;
        let mut others_riichi_declared = false;
        let mut cur_oya = 0;
        let mut jun = 0;
        let mut fuuro_num = 0;
        events.iter().for_each(|ev| match *ev {
            Event::StartKyoku { oya, scores, .. } => {
                stat.round += 1;
                cur_scores = scores;
                riichi_declared = false;
                riichi_accepted = false;
                others_riichi_declared = false;
                cur_oya = oya;
                if cur_oya == player_id {
                    stat.oya += 1;
                }
                jun = 0;
                fuuro_num = 0;
            }

            Event::Dahai { actor, .. } if actor == player_id => {
                jun += 1;
            }

            Event::Chi { actor, .. }
            | Event::Pon { actor, .. }
            | Event::Daiminkan { actor, .. }
                if actor == player_id =>
            {
                fuuro_num += 1;
            }

            Event::Reach { actor } => {
                if actor == player_id {
                    riichi_declared = true;
                    stat.riichi += 1;
                    stat.riichi_jun += jun;
                    if cur_oya == player_id {
                        stat.riichi_as_oya += 1;
                    }
                    if others_riichi_declared {
                        stat.chasing_riichi += 1;
                    }
                } else if riichi_declared {
                    stat.riichi_got_chased += 1;
                } else {
                    others_riichi_declared = true;
                }
            }

            Event::ReachAccepted { actor } => {
                cur_scores[actor as usize] -= 1000;
                if actor == player_id {
                    riichi_accepted = true;
                }
            }

            Event::Hora {
                actor,
                target,
                deltas,
                ..
            } => {
                let deltas = deltas.expect("deltas is required for analyzing");
                vec_add_assign(&mut cur_scores, &deltas);

                if actor == player_id {
                    let point = deltas[player_id as usize] as i64 - riichi_accepted as i64 * 1000;
                    stat.agari += 1;
                    stat.agari_jun += jun;
                    if cur_oya == player_id {
                        stat.agari_as_oya += 1;
                        stat.agari_point_oya += point;
                    } else {
                        stat.agari_point_ko += point;
                    }

                    if riichi_accepted {
                        stat.riichi_agari += 1;
                        stat.riichi_agari_jun += jun;
                        stat.riichi_agari_point += point;
                        stat.riichi_point += point;
                    } else if fuuro_num > 0 {
                        stat.fuuro_agari += 1;
                        stat.fuuro_agari_jun += jun;
                        stat.fuuro_agari_point += point;
                        stat.fuuro_point += point;
                    } else {
                        stat.dama_agari += 1;
                        stat.dama_agari_jun += jun;
                        stat.dama_agari_point += point;
                    }

                    if point >= Point::yakuman(cur_oya == player_id, 1).ron as i64 {
                        stat.yakuman += 1;
                    }
                } else if target == player_id {
                    let point = deltas[player_id as usize] as i64;
                    stat.houjuu += 1;
                    stat.houjuu_jun += jun;
                    if cur_oya == actor {
                        stat.houjuu_to_oya += 1;
                        stat.houjuu_point_to_oya += point;
                    } else {
                        stat.houjuu_point_to_ko += point;
                    }

                    if riichi_declared {
                        stat.riichi_houjuu += 1;
                        stat.riichi_point += point;
                    } else if fuuro_num > 0 {
                        stat.fuuro_houjuu += 1;
                        stat.fuuro_point += point;
                    }
                }
            }

            Event::Ryukyoku { deltas } => {
                let deltas = deltas.expect("deltas is required for analyzing");
                vec_add_assign(&mut cur_scores, &deltas);

                let point = deltas[player_id as usize] as i64;
                stat.ryukyoku += 1;
                stat.ryukyoku_point += point;
                if riichi_accepted {
                    stat.riichi_ryukyoku += 1;
                    stat.riichi_point += point - 1000;
                } else if fuuro_num > 0 {
                    stat.fuuro_point += point;
                }

                if point >= 8000 {
                    stat.nagashi_mangan += 1;
                }
            }

            Event::EndKyoku => {
                if fuuro_num > 0 {
                    stat.fuuro += 1;
                    stat.fuuro_num += fuuro_num;
                }
            }

            _ => (),
        });

        let rk = Rankings::new(cur_scores);

        // assume the sum of scores to be 100k
        let sum: i32 = cur_scores.iter().sum();
        if sum < 100_000 {
            cur_scores[rk.player_by_rank[0] as usize] += 100_000 - sum;
        }

        // assume the starting point to be 25000
        let final_score = cur_scores[player_id as usize];
        stat.point = final_score as i64 - 25000;
        if final_score < 0 {
            stat.tobi = 1;
        }

        let rank = rk.rank_by_player[player_id as usize];
        match rank {
            0 => stat.rank_1 = 1,
            1 => stat.rank_2 = 1,
            2 => stat.rank_3 = 1,
            _ => stat.rank_4 = 1,
        }

        stat
    }
}

#[pymethods]
impl Stat {
    #[staticmethod]
    #[pyo3(signature = (dir, player_name, disable_progress_bar=false))]
    pub fn from_dir(dir: &str, player_name: &str, disable_progress_bar: bool) -> Result<Self> {
        let bar = if disable_progress_bar {
            ProgressBar::hidden()
        } else {
            const TEMPLATE: &str = "{spinner:.cyan} [{elapsed_precise}] {pos} ({per_sec})";
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template(TEMPLATE)?.tick_chars(".oO°Oo*"))
        };
        bar.enable_steady_tick(Duration::from_millis(150));

        let stat = glob(&format!("{dir}/**/*.json"))?
            .chain(glob(&format!("{dir}/**/*.json.gz"))?)
            .par_bridge()
            .map(|path| {
                bar.inc(1);
                let path = path?;

                let raw_log = if path
                    .extension()
                    .is_some_and(|s| s.eq_ignore_ascii_case("gz"))
                {
                    io::read_to_string(GzDecoder::new(File::open(path)?))?
                } else {
                    io::read_to_string(File::open(path)?)?
                };

                let events = raw_log
                    .lines()
                    .map(json::from_str)
                    .collect::<Result<Vec<Event>, _>>()
                    .context("failed to parse log")?;

                match events.first() {
                    Some(Event::StartGame { names, .. }) => {
                        let log_stat = names
                            .iter()
                            .enumerate()
                            .filter(|&(_, name)| name == player_name)
                            .map(|(i, _)| Self::from_game(&events, i as u8))
                            .sum();
                        Ok(log_stat)
                    }
                    ev => bail!("first event is not start_game, got {ev:?}"),
                }
            })
            .sum::<Result<_>>()?;

        bar.abandon();
        Ok(stat)
    }

    #[staticmethod]
    pub fn from_log(log: &str, player_id: u8) -> Result<Self> {
        let events = log
            .lines()
            .map(json::from_str)
            .collect::<Result<Vec<Event>, _>>()
            .context("failed to parse log")?;
        Ok(Self::from_game(&events, player_id))
    }

    #[inline]
    #[must_use]
    pub const fn total_pt(&self, pts: [i64; 4]) -> i64 {
        self.rank_1 * pts[0] + self.rank_2 * pts[1] + self.rank_3 * pts[2] + self.rank_4 * pts[3]
    }
    #[inline]
    #[must_use]
    pub fn avg_pt(&self, pts: [i64; 4]) -> f64 {
        self.total_pt(pts) as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_rank(&self) -> f64 {
        self.avg_pt([1, 2, 3, 4])
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn rank_1_rate(&self) -> f64 {
        self.rank_1 as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn rank_2_rate(&self) -> f64 {
        self.rank_2 as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn rank_3_rate(&self) -> f64 {
        self.rank_3 as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn rank_4_rate(&self) -> f64 {
        self.rank_4 as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn tobi_rate(&self) -> f64 {
        self.tobi as f64 / self.game as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_game(&self) -> f64 {
        self.point as f64 / self.game as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_round(&self) -> f64 {
        self.point as f64 / self.round as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_agari(&self) -> f64 {
        (self.agari_point_ko + self.agari_point_oya) as f64 / self.agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_oya_agari(&self) -> f64 {
        self.agari_point_oya as f64 / self.agari_as_oya as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_ko_agari(&self) -> f64 {
        self.agari_point_ko as f64 / (self.agari - self.agari_as_oya) as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_riichi_agari(&self) -> f64 {
        self.riichi_agari_point as f64 / self.riichi_agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_fuuro_agari(&self) -> f64 {
        self.fuuro_agari_point as f64 / self.fuuro_agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_dama_agari(&self) -> f64 {
        self.dama_agari_point as f64 / self.dama_agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_ryukyoku(&self) -> f64 {
        self.ryukyoku_point as f64 / self.ryukyoku as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_agari_jun(&self) -> f64 {
        self.agari_jun as f64 / self.agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_riichi_agari_jun(&self) -> f64 {
        self.riichi_agari_jun as f64 / self.riichi_agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_fuuro_agari_jun(&self) -> f64 {
        self.fuuro_agari_jun as f64 / self.fuuro_agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_dama_agari_jun(&self) -> f64 {
        self.dama_agari_jun as f64 / self.dama_agari as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_houjuu(&self) -> f64 {
        (self.houjuu_point_to_ko + self.houjuu_point_to_oya) as f64 / self.houjuu as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_houjuu_to_oya(&self) -> f64 {
        self.houjuu_point_to_oya as f64 / self.houjuu_to_oya as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_point_per_houjuu_to_ko(&self) -> f64 {
        self.houjuu_point_to_ko as f64 / (self.houjuu - self.houjuu_to_oya) as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_houjuu_jun(&self) -> f64 {
        self.houjuu_jun as f64 / self.houjuu as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn agari_rate(&self) -> f64 {
        self.agari as f64 / self.round as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn houjuu_rate(&self) -> f64 {
        self.houjuu as f64 / self.round as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn riichi_rate(&self) -> f64 {
        self.riichi as f64 / self.round as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn fuuro_rate(&self) -> f64 {
        self.fuuro as f64 / self.round as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn ryukyoku_rate(&self) -> f64 {
        self.ryukyoku as f64 / self.round as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn agari_rate_after_riichi(&self) -> f64 {
        self.riichi_agari as f64 / self.riichi as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn houjuu_rate_after_riichi(&self) -> f64 {
        self.riichi_houjuu as f64 / self.riichi as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn chasing_riichi_rate(&self) -> f64 {
        self.chasing_riichi as f64 / self.riichi as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn riichi_chased_rate(&self) -> f64 {
        self.riichi_got_chased as f64 / self.riichi as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_riichi_jun(&self) -> f64 {
        self.riichi_jun as f64 / self.riichi as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_riichi_point(&self) -> f64 {
        self.riichi_point as f64 / self.riichi as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn agari_rate_as_oya(&self) -> f64 {
        self.agari_as_oya as f64 / self.oya as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn agari_as_oya_rate(&self) -> f64 {
        self.agari_as_oya as f64 / self.agari as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn houjuu_to_oya_rate(&self) -> f64 {
        self.houjuu_to_oya as f64 / self.houjuu as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_fuuro_num(&self) -> f64 {
        self.fuuro_num as f64 / self.fuuro as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn agari_rate_after_fuuro(&self) -> f64 {
        self.fuuro_agari as f64 / self.fuuro as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn houjuu_rate_after_fuuro(&self) -> f64 {
        self.fuuro_houjuu as f64 / self.fuuro as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn avg_fuuro_point(&self) -> f64 {
        self.fuuro_point as f64 / self.fuuro as f64
    }

    #[getter]
    #[inline]
    #[must_use]
    pub fn yakuman_rate(&self) -> f64 {
        self.yakuman as f64 / self.round as f64
    }
    #[getter]
    #[inline]
    #[must_use]
    pub fn nagashi_mangan_rate(&self) -> f64 {
        self.nagashi_mangan as f64 / self.round as f64
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

pub(crate) fn register_module(
    py: Python<'_>,
    prefix: &str,
    super_mod: &Bound<'_, PyModule>,
) -> PyResult<()> {
    let m = PyModule::new(py, "stat")?;
    m.add_class::<Stat>()?;
    add_submodule(py, prefix, super_mod, &m)
}
