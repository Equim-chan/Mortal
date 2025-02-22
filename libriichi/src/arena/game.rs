use super::board::{Board, BoardState, Poll};
use super::result::GameResult;
use crate::agent::BatchAgent;
use crate::mjai::EventExt;
use std::time::Duration;
use std::{array, mem};

use anyhow::{Result, ensure};
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::prelude::*;

pub struct BatchGame {
    /// 8 for hanchan and 4 for tonpuu
    pub length: u8,
    pub init_scores: [i32; 4],
    pub disable_progress_bar: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Index {
    /// For `Game` to find a specific `Agent` (game -> agent).
    pub agent_idx: usize,
    /// For `Agent` to find a specific player ID (agent -> game).
    pub player_id_idx: usize,
}

#[derive(Default)]
struct Game {
    length: u8,
    seed: (u64, u64),
    indexes: [Index; 4],

    oracle_obs_versions: [Option<u32>; 4],
    invisible_state_cache: [Option<Array2<f32>>; 4],

    last_reactions: [EventExt; 4], // cached for poll phase

    board: BoardState,
    kyoku: u8,
    honba: u8,
    kyotaku: u8,
    scores: [i32; 4],
    game_log: Vec<Vec<EventExt>>,

    kyoku_started: bool,
    ended: bool,
    /// Used in 西入 where the oya and another player get to 30000 at the same
    /// time, but the game continues because oya is not the top.
    ///
    /// As per [Tenhou's rule](https://tenhou.net/man/#RULE):
    ///
    /// > サドンデスルールは、30000点(供託未収)以上になった時点で終了、ただし親の
    /// > 連荘がある場合は連荘を優先する
    in_renchan: bool,
}

impl Game {
    /// Returns iff any player in the game can act or the game has ended.
    fn poll(&mut self, agents: &mut [Box<dyn BatchAgent>]) -> Result<()> {
        if self.ended {
            return Ok(());
        }

        if !self.kyoku_started {
            // after W4
            // or, after all-last
            //   and, oya is not in renchan (if oya is in renchan, it would already have been ended in the renchan owari check)
            //   and, anyone has more than 30k
            if self.kyoku >= self.length + 4
                || self.kyoku >= self.length
                    && !self.in_renchan
                    && self.scores.iter().any(|&s| s >= 30000)
            {
                self.ended = true;
                return Ok(());
            }

            let mut next_board = Board {
                kyoku: self.kyoku,
                honba: self.honba,
                kyotaku: self.kyotaku,
                scores: self.scores,
                ..Default::default()
            };
            next_board.init_from_seed(self.seed);
            self.board = next_board.into_state();
            self.kyoku_started = true;
        }

        let reactions = mem::take(&mut self.last_reactions);
        let poll = self.board.poll(reactions)?;
        match poll {
            Poll::InGame => {
                let ctx = self.board.agent_context();
                for (player_id, state) in ctx.player_states.iter().enumerate() {
                    if !state.last_cans().can_act() {
                        continue;
                    }

                    let invisible_state = self.oracle_obs_versions[player_id]
                        .map(|ver| self.board.encode_oracle_obs(player_id as u8, ver));
                    self.invisible_state_cache[player_id].clone_from(&invisible_state);

                    let idx = self.indexes[player_id];
                    agents[idx.agent_idx].set_scene(
                        idx.player_id_idx,
                        ctx.log,
                        state,
                        invisible_state,
                    )?;
                }
            }

            Poll::End => {
                self.kyoku_started = false;
                self.in_renchan = false;

                for idx in &self.indexes {
                    agents[idx.agent_idx].end_kyoku(idx.player_id_idx)?;
                }

                let kyoku_result = self.board.end();
                self.kyotaku = kyoku_result.kyotaku_left;
                self.scores = kyoku_result.scores;

                let logs = self.board.take_log();
                self.game_log.push(logs);

                let has_tobi = self.scores.iter().any(|&s| s < 0);
                if has_tobi {
                    self.ended = true;
                    return Ok(());
                }

                if kyoku_result.has_abortive_ryukyoku {
                    self.honba += 1;
                    return self.poll(agents);
                }

                if !kyoku_result.can_renchan {
                    self.kyoku += 1;
                    if kyoku_result.has_hora {
                        self.honba = 0;
                    } else {
                        self.honba += 1;
                    }
                    return self.poll(agents);
                }

                // renchan owari conditions:
                // 1. can renchan
                // 2. is at all-last
                // 3. oya has at least 30000
                // 4. oya is the top
                let oya = kyoku_result.kyoku as usize % 4;
                if kyoku_result.kyoku >= self.length - 1 && self.scores[oya] >= 30000 {
                    let top = kyoku_result
                        .scores
                        .iter()
                        .enumerate()
                        .min_by_key(|&(_, &s)| -s)
                        .map(|(i, _)| i)
                        .unwrap();
                    if top == oya {
                        self.ended = true;
                        return Ok(());
                    }
                }

                // renchan
                self.in_renchan = true;
                self.honba += 1;
                return self.poll(agents);
            }
        };

        Ok(())
    }

    fn commit(&mut self, agents: &mut [Box<dyn BatchAgent>]) -> Result<Option<GameResult>> {
        if self.ended {
            if self.kyotaku > 0 {
                *self.scores.iter_mut().min_by_key(|s| -**s).unwrap() += self.kyotaku as i32 * 1000;
            }

            let names = array::from_fn(|i| agents[self.indexes[i].agent_idx].name());
            let game_result = GameResult {
                names,
                scores: self.scores,
                seed: self.seed,
                game_log: mem::take(&mut self.game_log),
            };

            for idx in &self.indexes {
                agents[idx.agent_idx].end_game(idx.player_id_idx, &game_result)?;
            }
            return Ok(Some(game_result));
        }

        let ctx = self.board.agent_context();
        for (player_id, state) in ctx.player_states.iter().enumerate() {
            if !state.last_cans().can_act() {
                continue;
            }

            let invisible_state = self.invisible_state_cache[player_id].take();

            let idx = self.indexes[player_id];
            self.last_reactions[player_id] = agents[idx.agent_idx].get_reaction(
                idx.player_id_idx,
                ctx.log,
                state,
                invisible_state,
            )?;
        }

        Ok(None)
    }
}

impl BatchGame {
    pub const fn tenhou_hanchan(disable_progress_bar: bool) -> Self {
        Self {
            length: 8,
            init_scores: [25000; 4],
            disable_progress_bar,
        }
    }

    pub fn run(
        &self,
        agents: &mut [Box<dyn BatchAgent>],
        indexes: &[[Index; 4]],
        seeds: &[(u64, u64)],
    ) -> Result<Vec<GameResult>> {
        ensure!(!agents.is_empty());
        ensure!(!indexes.is_empty());
        ensure!(
            indexes.len() == seeds.len(),
            "expected `indexes.len() == seeds.len()`, got {} and {}",
            indexes.len(),
            seeds.len(),
        );

        let mut games = indexes
            .iter()
            .zip(seeds)
            .enumerate()
            .map(|(game_idx, (idxs, &seed))| {
                let mut oracle_obs_versions = [None; 4];
                for (i, idx) in idxs.iter().enumerate() {
                    agents[idx.agent_idx].start_game(idx.player_id_idx)?;
                    oracle_obs_versions[i] = agents[idx.agent_idx].oracle_obs_version();
                }

                let game = Box::new(Game {
                    length: self.length,
                    seed,
                    indexes: *idxs,
                    scores: self.init_scores,
                    oracle_obs_versions,
                    ..Default::default()
                });
                Ok((game_idx, game))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut game_results = vec![GameResult::default(); games.len()];
        let mut to_remove = vec![];
        let mut cycles = 0;
        let mut actions = 0;

        let bar = if self.disable_progress_bar {
            ProgressBar::hidden()
        } else {
            ProgressBar::new(games.len() as u64)
        };
        const TEMPLATE: &str =
            "{spinner:.cyan} {msg}\n[{elapsed_precise}] [{wide_bar}] {pos}/{len} {percent:>3}%";
        let style = ProgressStyle::with_template(TEMPLATE)?
            .tick_chars(".oO°Oo*")
            .progress_chars("#-");
        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(150));

        while !games.is_empty() {
            for (_, game) in &mut games {
                game.poll(agents)?;
            }

            for (idx_for_rm, (game_idx, game)) in games.iter_mut().enumerate() {
                if let Some(game_result) = game.commit(agents)? {
                    game_results[*game_idx] = game_result;
                    to_remove.push(idx_for_rm);
                }
            }

            for idx_for_rm in to_remove.drain(..).rev() {
                games.swap_remove(idx_for_rm);
                bar.inc(1);
            }

            cycles += 1;
            actions += games.len();

            let secs = bar.elapsed().as_secs_f64();
            bar.set_message(format!(
                "cycles: {cycles} ({:.3} cycle/s), actions: {actions} ({:.3} action/s)",
                cycles as f64 / secs,
                actions as f64 / secs,
            ));
        }
        bar.abandon();

        Ok(game_results)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::agent::Tsumogiri;

    #[test]
    fn tsumogiri() {
        let g = BatchGame::tenhou_hanchan(true);
        let mut agents = [
            Box::new(Tsumogiri::new_batched(&[0, 1, 2, 3]).unwrap()) as _,
            Box::new(Tsumogiri::new_batched(&[3, 2, 1, 0]).unwrap()) as _,
        ];
        let indexes = &[
            [
                Index {
                    agent_idx: 0,
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
                Index {
                    agent_idx: 1,
                    player_id_idx: 0,
                },
            ],
            [
                Index {
                    agent_idx: 1,
                    player_id_idx: 3,
                },
                Index {
                    agent_idx: 1,
                    player_id_idx: 2,
                },
                Index {
                    agent_idx: 0,
                    player_id_idx: 2,
                },
                Index {
                    agent_idx: 0,
                    player_id_idx: 3,
                },
            ],
        ];

        g.run(&mut agents, indexes, &[(1009, 0), (1021, 0)])
            .unwrap();
    }
}
