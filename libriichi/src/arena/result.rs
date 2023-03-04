use crate::mjai::{Event, EventExt};
use crate::rankings::Rankings;

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

#[derive(Clone, Copy)]
pub enum KyokuEndState {
    Passive = 0,
    Draw = 1,
    Win = 2,
    DealIn = 3,
}

impl GameResult {
    #[inline]
    pub fn rankings(&self) -> Rankings {
        Rankings::new(self.scores)
    }

    pub fn dump_json_log(&self) -> Result<String> {
        let mut v = vec![];

        let start_game = Event::StartGame {
            names: self.names.clone(),
            seed: Some(self.seed),
        };
        json::to_writer(&mut v, &start_game)?;
        v.push(b'\n');

        for ev in self.game_log.iter().flatten() {
            json::to_writer(&mut v, ev)?;
            v.push(b'\n');
        }

        json::to_writer(&mut v, &Event::EndGame)?;
        v.push(b'\n');

        Ok(String::from_utf8(v)?)
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
