use super::{BatchAgent, InvisibleState};
use crate::arena::GameResult;
use crate::mjai::EventExt;
use crate::state::PlayerState;
use std::mem;

use anyhow::{Context, Result, ensure};
use pyo3::intern;
use pyo3::prelude::*;
use serde_json as json;

pub struct MjaiLogBatchAgent {
    engine: PyObject,
    name: String,

    game_states: Vec<GameState>,
    reactions: Vec<Option<EventExt>>,
    reactions_idxs: Vec<usize>,
    evaluated: bool,
}

#[pyclass]
#[derive(Default)]
struct GameState {
    #[pyo3(get)]
    game_index: usize,
    #[pyo3(get)]
    state: PlayerState,
    #[pyo3(get)]
    events_json: String,
}

impl MjaiLogBatchAgent {
    pub fn new(engine: PyObject, player_ids: &[u8]) -> Result<Self> {
        ensure!(player_ids.iter().all(|&id| matches!(id, 0..=3)));

        let name = Python::with_gil(|py| {
            let obj = engine.bind_borrowed(py);
            for method in ["react_batch", "start_game", "end_kyoku", "end_game"] {
                ensure!(
                    obj.getattr(method)?.is_callable(),
                    "missing method {method}",
                );
            }

            let name = obj.getattr("name")?.extract()?;
            obj.call_method1("set_player_ids", (player_ids,))?;
            Ok(name)
        })?;

        let size = player_ids.len();
        Ok(Self {
            engine,
            name,

            game_states: vec![],
            reactions: vec![],
            reactions_idxs: vec![0; size],
            evaluated: false,
        })
    }

    fn evaluate(&mut self) -> Result<()> {
        if self.game_states.is_empty() {
            return Ok(());
        }

        let raw_reactions: Vec<String> = Python::with_gil(|py| {
            let game_states = mem::take(&mut self.game_states);
            self.engine
                .bind_borrowed(py)
                .call_method1(intern!(py, "react_batch"), (game_states,))
                .context("failed to execute `react_batch` on Python engine")?
                .extract()
                .context("failed to extract to Rust type")
        })?;
        self.reactions.clear();
        for s in raw_reactions {
            let ev = json::from_str(&s)?;
            self.reactions.push(Some(ev));
        }

        Ok(())
    }
}

impl BatchAgent for MjaiLogBatchAgent {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_scene(
        &mut self,
        index: usize,
        log: &[EventExt],
        state: &PlayerState,
        _: Option<InvisibleState>,
    ) -> Result<()> {
        self.evaluated = false;

        let game_state = GameState {
            game_index: index,
            state: state.clone(),
            events_json: json::to_string(&log)?,
        };
        self.game_states.push(game_state);
        self.reactions_idxs[index] = self.game_states.len() - 1;

        Ok(())
    }

    fn get_reaction(
        &mut self,
        index: usize,
        _: &[EventExt],
        _: &PlayerState,
        _: Option<InvisibleState>,
    ) -> Result<EventExt> {
        if !self.evaluated {
            self.evaluate()?;
            self.evaluated = true;
        }

        let reactions_idx = self.reactions_idxs[index];
        let event = self.reactions[reactions_idx]
            .take()
            .context("take after take")?;

        Ok(event)
    }

    fn start_game(&mut self, index: usize) -> Result<()> {
        Python::with_gil(|py| {
            self.engine
                .bind_borrowed(py)
                .call_method1(intern!(py, "start_game"), (index,))?;
            Ok(())
        })
    }

    fn end_kyoku(&mut self, index: usize) -> Result<()> {
        Python::with_gil(|py| {
            self.engine
                .bind_borrowed(py)
                .call_method1(intern!(py, "end_kyoku"), (index,))?;
            Ok(())
        })
    }

    fn end_game(&mut self, index: usize, game_result: &GameResult) -> Result<()> {
        Python::with_gil(|py| {
            self.engine
                .bind_borrowed(py)
                .call_method1(intern!(py, "end_game"), (index, game_result.scores))?;
            Ok(())
        })
    }
}
