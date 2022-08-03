use super::game::{BatchGame, Index};
use super::result::GameResult;
use crate::agent::{AkochanAgent, BatchAgent, MortalBatchAgent};
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter;
use std::path::PathBuf;
use std::time::Duration;

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
pub struct TwoVsTwo {
    pub disable_progress_bar: bool,
    pub log_dir: Option<String>,
}

#[pymethods]
impl TwoVsTwo {
    #[new]
    #[args("*", disable_progress_bar = "false", log_dir = "None")]
    const fn new(disable_progress_bar: bool, log_dir: Option<String>) -> Self {
        Self {
            disable_progress_bar,
            log_dir,
        }
    }

    #[pyo3(text_signature = "(challenger, champion, seed_start, seed_count)")]
    pub fn py_vs_py(
        &self,
        challenger: PyObject,
        champion: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<()> {
        // `allow_threads` is required, otherwise it will block python GC to
        // run, leading to memory leaks, since this function is doing long
        // tasks.
        py.allow_threads(move || {
            self.run_batch(
                |player_ids| MortalBatchAgent::new(challenger, player_ids),
                |player_ids| MortalBatchAgent::new(champion, player_ids),
                seed_start,
                seed_count,
            )?;
            Ok(())
        })
    }

    #[pyo3(text_signature = "($self, engine, seed_start, seed_count)")]
    pub fn ako_vs_py(
        &self,
        engine: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<()> {
        py.allow_threads(move || {
            self.run_batch(
                AkochanAgent::new_batched,
                |player_ids| MortalBatchAgent::new(engine, player_ids),
                seed_start,
                seed_count,
            )?;
            Ok(())
        })
    }

    #[pyo3(text_signature = "($self, engine, seed_start, seed_count)")]
    pub fn py_vs_ako(
        &self,
        engine: PyObject,
        seed_start: (u64, u64),
        seed_count: u64,
        py: Python<'_>,
    ) -> Result<()> {
        py.allow_threads(move || {
            self.run_batch(
                |player_ids| MortalBatchAgent::new(engine, player_ids),
                AkochanAgent::new_batched,
                seed_start,
                seed_count,
            )?;
            Ok(())
        })
    }

    #[pyo3(text_signature = "($self, engine, seed, split)")]
    pub fn py_vs_ako_one(
        &self,
        engine: PyObject,
        seed: (u64, u64),
        split: usize,
        py: Python<'_>,
    ) -> Result<()> {
        py.allow_threads(move || {
            self.run_one(
                |player_ids| MortalBatchAgent::new(engine, player_ids),
                AkochanAgent::new_batched,
                seed,
                split,
            )?;
            Ok(())
        })
    }
}

impl TwoVsTwo {
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
            seed_count * 2,
        );

        let seeds: Vec<_> = (seed_start.0..seed_start.0 + seed_count)
            .flat_map(|seed| iter::repeat((seed, seed_start.1)).take(2))
            .collect();

        let challenger_player_ids: Vec<_> = [
            0, 2, // A
            1, 3, // B
        ]
        .into_iter()
        .cycle()
        .take(seed_count as usize * 4)
        .collect();
        let champion_player_ids: Vec<_> = [
            1, 3, // A
            0, 2, // B
        ]
        .into_iter()
        .cycle()
        .take(seed_count as usize * 4)
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
                    make_idx_group([0, 1, 0, 1]),
                    // split B
                    make_idx_group([1, 0, 1, 0]),
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
            const TEMPLATE: &str = "{spinner:.cyan} steps: {msg}\n[{elapsed_precise}] [{wide_bar}] {pos}/{len} {percent:>3}%";
            bar.set_style(
                ProgressStyle::with_template(TEMPLATE)?
                    .tick_chars(".oOo")
                    .progress_chars("#-"),
            );
            bar.enable_steady_tick(Duration::from_millis(150));

            results
                .par_iter()
                .progress_with(bar)
                .enumerate()
                .try_for_each(|(i, game_result)| {
                    let split_name = ["a", "b"][i % 2];
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

    pub fn run_one<T, R, TA, RA>(
        &self,
        new_challenger_agent: T,
        new_champion_agent: R,
        seed: (u64, u64),
        split: usize, // must be within 0..2
    ) -> Result<GameResult>
    where
        T: FnOnce(&[u8]) -> Result<TA>,
        R: FnOnce(&[u8]) -> Result<RA>,
        TA: BatchAgent + 'static,
        RA: BatchAgent + 'static,
    {
        if let Some(dir) = &self.log_dir {
            fs::create_dir_all(dir)?;
        }

        log::info!(
            "seed: {} w/ {}, split: {}, start 1 hanchan",
            seed.0,
            seed.1,
            split
        );

        let challenger_player_ids = if split == 0 { [0, 2] } else { [1, 3] };
        let champion_player_ids = if split == 0 { [1, 3] } else { [0, 2] };

        let mut agents: [Box<dyn BatchAgent>; 2] = [
            Box::new(new_challenger_agent(&challenger_player_ids)?),
            Box::new(new_champion_agent(&champion_player_ids)?),
        ];
        let batch_game = BatchGame::tenhou_hanchan(self.disable_progress_bar);

        let indexes = if split == 0 {
            [[
                Index {
                    agent_idx: 0,
                    player_id_idx: 0,
                },
                Index {
                    agent_idx: 1,
                    player_id_idx: 0,
                },
                Index {
                    agent_idx: 0,
                    player_id_idx: 1,
                },
                Index {
                    agent_idx: 1,
                    player_id_idx: 1,
                },
            ]]
        } else {
            [[
                Index {
                    agent_idx: 1,
                    player_id_idx: 0,
                },
                Index {
                    agent_idx: 0,
                    player_id_idx: 0,
                },
                Index {
                    agent_idx: 1,
                    player_id_idx: 1,
                },
                Index {
                    agent_idx: 0,
                    player_id_idx: 1,
                },
            ]]
        };

        let results = batch_game.run(&mut agents, &indexes, &[seed])?;

        if let Some(dir) = &self.log_dir {
            log::info!("dumping game logs");

            let split_name = ["a", "b"][split];
            let filename: PathBuf = [dir, &format!("{}_{}_{split_name}.json.gz", seed.0, seed.1)]
                .iter()
                .collect();

            let log = results[0].dump_json_log()?;
            let mut comp = GzEncoder::new(log.as_bytes(), Compression::best());
            let mut data = vec![];
            comp.read_to_end(&mut data)?;

            let mut f = File::create(filename)?;
            f.write_all(&data)?;
            f.sync_all()?;
        }

        Ok(results.into_iter().next().unwrap())
    }
}
