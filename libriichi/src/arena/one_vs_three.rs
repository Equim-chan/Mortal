use super::game::{BatchGame, Index};
use super::result::GameResult;
use crate::agent::{AkochanAgent, BatchAgent, MortalBatchAgent};
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter;
use std::path::PathBuf;

use anyhow::Result;
use flate2::read::GzEncoder;
use flate2::Compression;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use pyo3::prelude::*;
use rayon::prelude::*;

#[pyclass]
#[pyo3(text_signature = "(
    *,
    disable_progress_bar = False,
    log_dir = None,
)")]
#[derive(Clone, Default)]
pub struct OneVsThree {
    pub disable_progress_bar: bool,
    pub log_dir: Option<String>,
}

#[pymethods]
impl OneVsThree {
    #[new]
    #[args("*", disable_progress_bar = "false", log_dir = "None")]
    const fn new(disable_progress_bar: bool, log_dir: Option<String>) -> Self {
        Self {
            disable_progress_bar,
            log_dir,
        }
    }

    /// Returns the rankings of the challenger.
    #[pyo3(text_signature = "($self, challenger, champion, seed_start, seed_count)")]
    pub fn py_vs_py(
        &self,
        challenger: PyObject,
        champion: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        // `allow_threads` is required, otherwise it will block python GC to
        // run, leading to memory leaks, since this function is doing long
        // tasks.
        py.allow_threads(move || {
            let results = self.run_batch(
                |player_ids| MortalBatchAgent::new(challenger, player_ids),
                |player_ids| MortalBatchAgent::new(champion, player_ids),
                seed_start,
                seed_count,
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }

    /// Returns the rankings of the challenger (akochan in this case).
    #[pyo3(text_signature = "($self, engine, seed_start, seed_count)")]
    pub fn ako_vs_py(
        &self,
        engine: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        py.allow_threads(move || {
            let results = self.run_batch(
                AkochanAgent::new_batched,
                |player_ids| MortalBatchAgent::new(engine, player_ids),
                seed_start,
                seed_count,
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }

    /// Returns the rankings of the challenger (python agent in this case).
    #[pyo3(text_signature = "($self, engine, seed_start, seed_count)")]
    pub fn py_vs_ako(
        &self,
        engine: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        py.allow_threads(move || {
            let results = self.run_batch(
                |player_ids| MortalBatchAgent::new(engine, player_ids),
                AkochanAgent::new_batched,
                seed_start,
                seed_count,
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }

    /// Returns the rankings of the challenger (akochan in this case).
    #[pyo3(text_signature = "($self, seed_start, seed_count)")]
    pub fn ako_vs_ako(
        &self,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<[i32; 4]> {
        py.allow_threads(move || {
            let results = self.run_batch(
                AkochanAgent::new_batched,
                AkochanAgent::new_batched,
                seed_start,
                seed_count,
            )?;

            let mut rankings = [0; 4];
            for (i, result) in results.iter().enumerate() {
                let rank = result.rankings().rank_by_player[i % 4];
                rankings[rank as usize] += 1;
            }
            Ok(rankings)
        })
    }
}

impl OneVsThree {
    pub fn run_batch<C, M, CA, MA>(
        &self,
        new_challenger_agent: C,
        new_champion_agent: M,
        seed_start: (u64, u64),
        seed_count: u64,
    ) -> Result<Vec<GameResult>>
    where
        C: FnOnce(&[u8]) -> Result<CA>,
        M: FnOnce(&[u8]) -> Result<MA>,
        CA: BatchAgent + 'static,
        MA: BatchAgent + 'static,
    {
        if let Some(dir) = &self.log_dir {
            fs::create_dir_all(dir)?;
        }

        log::info!(
            "seed: [{}, {}) w/ {}, start {} groups, {} hanchans",
            seed_start.0,
            seed_start.0 + seed_count,
            seed_start.1,
            seed_count,
            seed_count * 4,
        );

        let seeds: Vec<_> = (seed_start.0..seed_start.0 + seed_count)
            .flat_map(|seed| iter::repeat((seed, seed_start.1)).take(4))
            .collect();

        let challenger_player_ids: Vec<_> = (0..4).cycle().take(seed_count as usize * 4).collect();
        let champion_player_ids: Vec<_> = [
            1, 2, 3, // A
            0, 2, 3, // B
            0, 1, 3, // C
            0, 1, 2, // D
        ]
        .into_iter()
        .cycle()
        .take(seed_count as usize * 4 * 3)
        .collect();

        let mut agents: [Box<dyn BatchAgent>; 2] = [
            Box::new(new_challenger_agent(&challenger_player_ids)?),
            Box::new(new_champion_agent(&champion_player_ids)?),
        ];
        let batch_game = BatchGame::tenhou_hanchan(self.disable_progress_bar);

        let mut challenger_idx = 0;
        let mut champion_idx = 0;
        let mut make_idx_group = |agent_idxs: [usize; 4]| {
            let mut idx_group = [Index::default(); 4];
            for (agent_idx, idx_item) in agent_idxs.into_iter().zip(&mut idx_group) {
                let player_id_idx = if agent_idx == 0 {
                    &mut challenger_idx
                } else {
                    &mut champion_idx
                };
                *idx_item = Index {
                    agent_idx,
                    player_id_idx: *player_id_idx,
                };
                *player_id_idx += 1;
            }
            idx_group
        };
        let indexes: Vec<_> = (0..seed_count)
            .flat_map(|_| {
                [
                    // split A
                    make_idx_group([0, 1, 1, 1]),
                    // split B
                    make_idx_group([1, 0, 1, 1]),
                    // split C
                    make_idx_group([1, 1, 0, 1]),
                    // split D
                    make_idx_group([1, 1, 1, 0]),
                ]
            })
            .collect();

        let results = batch_game.run(&mut agents, &indexes, &seeds)?;

        if let Some(dir) = &self.log_dir {
            log::info!("dumping game logs");

            let bar = if self.disable_progress_bar {
                ProgressBar::hidden()
            } else {
                ProgressBar::new(seed_count * 4)
            };
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.cyan} [{elapsed_precise}] [{wide_bar}] {pos}/{len} {percent:>3}%")
                    .tick_chars(".oOo")
                    .progress_chars("#-"),
            );
            bar.enable_steady_tick(150);

            results
                .par_iter()
                .progress_with(bar)
                .enumerate()
                .try_for_each(|(i, game_result)| {
                    let split_name = ["a", "b", "c", "d"][i % 4];
                    let filename: PathBuf = [
                        dir,
                        &format!(
                            "{}_{}_{split_name}.json.gz",
                            game_result.seed.0, game_result.seed.1,
                        ),
                    ]
                    .iter()
                    .collect();

                    let log = game_result.dump_json_log()?;
                    let mut comp = GzEncoder::new(log.as_bytes(), Compression::best());
                    let mut data = vec![];
                    comp.read_to_end(&mut data)?;

                    let mut f = File::create(filename)?;
                    f.write_all(&data)?;
                    f.sync_all()?;

                    anyhow::Ok(())
                })?;
        }

        Ok(results)
    }
}
