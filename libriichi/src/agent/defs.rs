use crate::arena::GameResult;
use crate::mjai::EventExt;
use crate::state::PlayerState;

use anyhow::Result;
use ndarray::prelude::*;

pub type InvisibleState = Array2<f32>;

/// `react` provides various choices for input, the implementor may choose
/// one or many of them to produce the result.
///
/// The caller SHOULD call `react` only when `cans.can_act()` holds.
pub trait Agent {
    fn name(&self) -> String;
    fn oracle_obs_version(&self) -> Option<u32> {
        None
    }

    fn react(
        &mut self,
        log: &[EventExt],
        state: &PlayerState,
        invisible_state: Option<InvisibleState>,
    ) -> Result<EventExt>;

    fn start_game(&mut self) -> Result<()> {
        Ok(())
    }
    fn end_kyoku(&mut self) -> Result<()> {
        Ok(())
    }
    fn end_game(&mut self, game_result: &GameResult) -> Result<()> {
        let _ = game_result;
        Ok(())
    }
}

pub trait BatchAgent {
    fn name(&self) -> String;
    fn oracle_obs_version(&self) -> Option<u32> {
        None
    }

    fn set_scene(
        &mut self,
        index: usize,
        log: &[EventExt],
        state: &PlayerState,
        invisible_state: Option<InvisibleState>,
    ) -> Result<()>;

    fn get_reaction(
        &mut self,
        index: usize,
        log: &[EventExt],
        state: &PlayerState,
        invisible_state: Option<InvisibleState>,
    ) -> Result<EventExt>;

    fn start_game(&mut self, index: usize) -> Result<()> {
        let _ = index;
        Ok(())
    }

    fn end_kyoku(&mut self, index: usize) -> Result<()> {
        let _ = index;
        Ok(())
    }

    fn end_game(&mut self, index: usize, game_result: &GameResult) -> Result<()> {
        let _ = index;
        let _ = game_result;
        Ok(())
    }
}
