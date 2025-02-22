use super::{Agent, BatchAgent};
use crate::arena::GameResult;
use crate::mjai::EventExt;
use crate::state::PlayerState;

use anyhow::{Context, Result, ensure};
use ndarray::prelude::*;

pub struct BatchifiedAgent<A>
where
    A: Agent,
{
    inner: Vec<A>,
    last_actions: Vec<Option<EventExt>>,
}

impl<A> BatchifiedAgent<A>
where
    A: Agent,
{
    /// Example:
    ///
    /// ```rust
    /// use riichi::agent::{BatchAgent, BatchifiedAgent, Tsumogiri};
    ///
    /// let batch_tsumogiri = BatchifiedAgent::new(|i| Ok(Tsumogiri(i)), &[0, 1, 2, 3]).unwrap();
    /// fn impl_test<T: BatchAgent>(_: T) {}
    /// impl_test(batch_tsumogiri);
    /// ```
    pub fn new<F>(new_fn: F, player_ids: &[u8]) -> Result<Self>
    where
        F: FnMut(u8) -> Result<A>,
    {
        ensure!(!player_ids.is_empty());

        let inner = player_ids
            .iter()
            .copied()
            .map(new_fn)
            .collect::<Result<_>>()?;
        Ok(Self {
            inner,
            last_actions: vec![None; player_ids.len()],
        })
    }
}

impl<A> BatchAgent for BatchifiedAgent<A>
where
    A: Agent,
{
    #[inline]
    fn name(&self) -> String {
        self.inner[0].name()
    }

    #[inline]
    fn oracle_obs_version(&self) -> Option<u32> {
        self.inner[0].oracle_obs_version()
    }

    #[inline]
    fn set_scene(
        &mut self,
        index: usize,
        log: &[EventExt],
        state: &PlayerState,
        invisible_features: Option<Array2<f32>>,
    ) -> Result<()> {
        self.last_actions[index] = Some(self.inner[index].react(log, state, invisible_features)?);
        Ok(())
    }

    #[inline]
    fn get_reaction(
        &mut self,
        index: usize,
        _: &[EventExt],
        _: &PlayerState,
        _: Option<Array2<f32>>,
    ) -> Result<EventExt> {
        self.last_actions[index]
            .take()
            .context("`get_reaction` without `set_scene`")
    }

    #[inline]
    fn start_game(&mut self, index: usize) -> Result<()> {
        self.inner[index].start_game()
    }

    #[inline]
    fn end_kyoku(&mut self, index: usize) -> Result<()> {
        self.inner[index].end_kyoku()
    }

    #[inline]
    fn end_game(&mut self, index: usize, game_result: &GameResult) -> Result<()> {
        self.inner[index].end_game(game_result)
    }
}
