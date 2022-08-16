use crate::mjai::{Event, EventExt};
use std::array;

use anyhow::Result;
use serde_json as json;

#[derive(Debug, Clone)]
pub struct KyokuResult {
    pub kyoku: u8,
    pub honba: u8,
    pub can_renchan: bool,
    pub has_hora: bool,
    pub has_abortive_ryukyoku: bool,
    pub kyotaku_left: u8,
    pub scores: [i32; 4],
}

#[derive(Debug, Clone, Default)]
pub struct GameResult {
    pub names: [String; 4],
    pub scores: [i32; 4],
    pub seed: (u64, u64),
    pub game_log: Vec<Vec<EventExt>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rankings {
    pub player_by_rank: [u8; 4],
    pub rank_by_player: [u8; 4],
}

#[derive(Clone, Copy)]
pub enum KyokuEndState {
    Passive = 0,
    Draw = 1,
    Win = 2,
    DealIn = 3,
}

impl GameResult {
    pub fn rankings(&self) -> Rankings {
        let mut player_by_rank = [0; 4];
        let mut rank_by_player = [0; 4];

        let mut v: [_; 4] = array::from_fn(|id| (id, self.scores[id]));
        v.sort_by_key(|(_, s)| -s);
        for (rank, (id, _)) in v.into_iter().enumerate() {
            player_by_rank[rank] = id as u8;
            rank_by_player[id] = rank as u8;
        }

        Rankings {
            player_by_rank,
            rank_by_player,
        }
    }

    pub fn dump_json_log(&self) -> Result<String> {
        let mut ret = json::to_string(&Event::StartGame {
            names: self.names.clone(),
            seed: Some(self.seed),
        })? + "\n";

        for kyoku in &self.game_log {
            for ev in kyoku {
                ret += &(json::to_string(ev)? + "\n");
            }
        }

        ret += &(json::to_string(&Event::EndGame)? + "\n");
        Ok(ret)
    }

    pub fn kyoku_end_states(&self, perspective: u8) -> Vec<KyokuEndState> {
        self.game_log
            .iter()
            .map(|log| {
                let mut ret = KyokuEndState::Passive;
                for ev in log.iter().rev() {
                    match ev.event {
                        Event::EndKyoku => continue,
                        Event::Ryukyoku { .. } => {
                            ret = KyokuEndState::Draw;
                        }
                        Event::Hora { actor, target, .. } => {
                            if actor == perspective {
                                ret = KyokuEndState::Win;
                            } else if target == perspective {
                                ret = KyokuEndState::DealIn;
                            } else {
                                continue;
                            }
                        }
                        _ => (),
                    };
                    break;
                }
                ret
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rankings() {
        let mut res = GameResult {
            scores: [25000, 25000, 30000, 20000],
            ..Default::default()
        };
        let r = res.rankings();
        assert_eq!(r.player_by_rank, [2, 0, 1, 3]);
        assert_eq!(r.rank_by_player, [1, 2, 0, 3]);
        *res.scores.iter_mut().min_by_key(|s| -**s).unwrap() = 0; // used in game end kyotaku give out
        assert_eq!(res.scores, [25000, 25000, 0, 20000]);

        res.scores = [25000, 25000, 25000, 25000];
        let r = res.rankings();
        assert_eq!(r.player_by_rank, [0, 1, 2, 3]);
        assert_eq!(r.rank_by_player, [0, 1, 2, 3]);
        *res.scores.iter_mut().min_by_key(|s| -**s).unwrap() = 0;
        assert_eq!(res.scores, [0, 25000, 25000, 25000]);

        res.scores = [18000, 32000, 32000, 18000];
        let r = res.rankings();
        assert_eq!(r.player_by_rank, [1, 2, 0, 3]);
        assert_eq!(r.rank_by_player, [2, 0, 1, 3]);
        *res.scores.iter_mut().min_by_key(|s| -**s).unwrap() = 0;
        assert_eq!(res.scores, [18000, 0, 32000, 18000]);

        res.scores = [32000, 18000, 18000, 32000];
        let r = res.rankings();
        assert_eq!(r.player_by_rank, [0, 3, 1, 2]);
        assert_eq!(r.rank_by_player, [0, 2, 3, 1]);
        *res.scores.iter_mut().min_by_key(|s| -**s).unwrap() = 0;
        assert_eq!(res.scores, [0, 18000, 18000, 32000]);

        res.scores = [0, 100000, 0, 0];
        let r = res.rankings();
        assert_eq!(r.player_by_rank, [1, 0, 2, 3]);
        assert_eq!(r.rank_by_player, [1, 0, 2, 3]);
        *res.scores.iter_mut().min_by_key(|s| -**s).unwrap() = 0;
        assert_eq!(res.scores, [0; 4]);
    }
}
