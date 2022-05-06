use super::board::{Board, BoardState, Suspension};
use super::result::GameResult;
use crate::agent::BatchAgent;
use crate::mjai::EventExt;
use std::collections::VecDeque;
use std::mem;

use anyhow::{ensure, Result};
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::prelude::*;

pub struct BatchGame {
    // 8 for hanchan and 4 for tonpuu
    pub length: u8,
    pub init_scores: [i32; 4],
    pub disable_progress_bar: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Index {
    // For `Game` to find a specific `Agent`.
    pub agent_idx: usize,
    // For `Agent` to find a specific player ID accordingly.
    pub player_id_idx: usize,
}

#[derive(Default)]
struct Game {
    length: u8,
    seed: (u64, u64),
    indexes: [Index; 4],

    need_invisible_state: [bool; 4],
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
    // Used in 西入 where the oya and another player get to 30000 at the same
    // time, but the game continues because oya is not the top.
    //
    // As per [Tenhou's rule](https://tenhou.net/man/#RULE):
    //
    // > サドンデスルールは、30000点(供託未収)以上になった時点で終了、ただし親の
    // > 連荘がある場合は連荘を優先する
    in_renchan: bool,
}

impl Game {
    fn poll(&mut self, agents: &mut [Box<dyn BatchAgent>]) -> Result<()> {
        if self.ended {
            return Ok(());
        }

        if !self.kyoku_started {
            if self.kyoku >= self.length + 4 // no 北入
                || self.kyoku >= self.length // in 西入
                    && !self.in_renchan // oya is not in renchan
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
        let suspension = self.board.step_forward(reactions)?;
        match suspension {
            Suspension::InGame => {
                let ctx = self.board.agent_context();
                for (player_id, state) in ctx.player_states.iter().enumerate() {
                    if !state.last_cans().can_act() {
                        continue;
                    }

                    let invisible_state = self.need_invisible_state[player_id]
                        .then(|| self.board.encode_oracle_obs(player_id as u8));
                    self.invisible_state_cache[player_id] = invisible_state.clone();

                    let idx = self.indexes[player_id];
                    agents[idx.agent_idx].set_scene(
                        idx.player_id_idx,
                        ctx.log,
                        state,
                        invisible_state,
                    )?;
                }
            }

            Suspension::End => {
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
                    return Ok(());
                }

                if !kyoku_result.can_renchan {
                    self.kyoku += 1;
                    if kyoku_result.has_hora {
                        self.honba = 0;
                    } else {
                        self.honba += 1;
                    }
                    return Ok(());
                }

                // renchan owari
                //
                // Conditions:
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
            }
        };

        Ok(())
    }

    fn commit(&mut self, agents: &mut [Box<dyn BatchAgent>]) -> Result<Option<GameResult>> {
        if self.ended {
            if self.kyotaku > 0 {
                *self.scores.iter_mut().min_by_key(|s| -**s).unwrap() += self.kyotaku as i32 * 1000;
            }

            let names = [
                agents[self.indexes[0].agent_idx].name(),
                agents[self.indexes[1].agent_idx].name(),
                agents[self.indexes[2].agent_idx].name(),
                agents[self.indexes[3].agent_idx].name(),
            ];
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
        indexes: &[Index],
        seeds: &[(u64, u64)],
    ) -> Result<Vec<GameResult>> {
        ensure!(!agents.is_empty(), "`agents` is empty");
        ensure!(!indexes.is_empty(), "`indexes` is empty");
        ensure!(
            indexes.len() % 4 == 0,
            "`indexes.len()` ({}) must be divisible by 4",
            indexes.len(),
        );
        ensure!(
            indexes.len() == seeds.len() * 4,
            "`indexes.len()` ({}) must be 4 times the `seeds.len()` ({})",
            indexes.len(),
            seeds.len(),
        );

        let mut games = indexes
            .chunks_exact(4)
            .zip(seeds)
            .enumerate()
            .map(|(game_idx, (idxs, &seed))| {
                let mut need_invisible_state = [false; 4];
                for (i, idx) in idxs.iter().enumerate() {
                    agents[idx.agent_idx].start_game(idx.player_id_idx)?;
                    need_invisible_state[i] = agents[idx.agent_idx].need_oracle_obs();
                }

                let game = Box::new(Game {
                    length: self.length,
                    seed,
                    indexes: [idxs[0], idxs[1], idxs[2], idxs[3]],
                    scores: self.init_scores,
                    need_invisible_state,
                    ..Default::default()
                });

                Ok((game_idx, game))
            })
            .collect::<Result<VecDeque<_>>>()?;

        let mut records = Vec::with_capacity(games.len());
        records.resize_with(games.len(), Default::default);
        let mut to_remove = Vec::with_capacity(games.len());
        let mut steps = 0; // for stats only

        let bar = if self.disable_progress_bar {
            ProgressBar::hidden()
        } else {
            ProgressBar::new(games.len() as u64)
        };
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} steps: {msg}\n[{elapsed_precise}] [{wide_bar}] {pos}/{len} {percent:>3}%")
                .tick_chars(".oOo")
                .progress_chars("#-"),
        );
        bar.enable_steady_tick(150);

        while !games.is_empty() {
            for (_, game) in &mut games {
                loop {
                    game.poll(agents)?;
                    if game.ended || game.kyoku_started {
                        break;
                    }
                }
            }

            for (idx_for_rm, (game_idx, game)) in games.iter_mut().enumerate() {
                if let Some(record) = game.commit(agents)? {
                    records[*game_idx] = record;
                    to_remove.push(idx_for_rm);
                    bar.inc(1);
                }
            }
            for idx_for_rm in to_remove.drain(..).rev() {
                games.remove(idx_for_rm);
            }

            steps += 1;
            bar.set_message(format!(
                "{steps} ({:.3} step/s)",
                steps as f64 / bar.elapsed().as_secs_f64(),
            ));
        }
        bar.abandon();

        Ok(records)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::agent::{BatchAgent, Tsumogiri};

    #[test]
    fn tsumogiri() {
        let g = BatchGame::tenhou_hanchan(true);
        let mut agents: Vec<Box<dyn BatchAgent>> = vec![
            Box::new(Tsumogiri::new_batched(&[0, 1, 2, 3]).unwrap()),
            Box::new(Tsumogiri::new_batched(&[3, 2, 1, 0]).unwrap()),
        ];
        let indexes = &[
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
        ];

        g.run(&mut agents, indexes, &[(1009, 0), (1021, 0)])
            .unwrap();
    }
}
