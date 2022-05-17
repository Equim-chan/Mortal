use super::{Agent, BatchifiedAgent, InvisibleState};
use crate::mjai::{Event, EventExt};
use crate::state::PlayerState;

use anyhow::{Context, Result};

/// `Tsumogiri` always performs tsumogiri in all case and will not emit any
/// action other than discard.
pub struct Tsumogiri(pub u8);

impl Tsumogiri {
    pub fn new_batched(player_ids: &[u8]) -> Result<BatchifiedAgent<Self>> {
        BatchifiedAgent::new(|id| Ok(Self(id)), player_ids)
    }
}

impl Agent for Tsumogiri {
    fn name(&self) -> String {
        "tsumogiri".to_owned()
    }

    fn react(
        &mut self,
        _: &[EventExt],
        state: &PlayerState,
        _: Option<InvisibleState>,
    ) -> Result<EventExt> {
        let ev = if state.last_cans().can_discard {
            Event::Dahai {
                actor: self.0,
                pai: state.last_self_tsumo().context("last tsumo is empty")?,
                tsumogiri: true,
            }
        } else {
            Event::None
        };
        Ok(EventExt::no_meta(ev))
    }
}
