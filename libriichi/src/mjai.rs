use crate::tile::Tile;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, skip_serializing_none};

/// Describes an event in mjai format.
///
/// Mjai protocol was originally defined in
/// <https://gimite.net/pukiwiki/index.php?Mjai%20%E9%BA%BB%E9%9B%80AI%E5%AF%BE%E6%88%A6%E3%82%B5%E3%83%BC%E3%83%90>.
/// This implementation does not contain the full specs defined in the original
/// one, and it has some extensions added.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derivative::Derivative)]
#[derivative(Default)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    #[derivative(Default)]
    None,

    StartGame {
        #[serde(default)]
        names: [String; 4],

        // See https://github.com/jonasbb/serde_with/issues/185 for the reason
        // for the serde(default).
        /// Consists of (nonce, key).
        #[serde(default)]
        seed: Option<(u64, u64)>,
    },
    StartKyoku {
        bakaze: Tile,
        dora_marker: Tile,
        /// Counts from 1
        kyoku: u8,
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        tehais: [[Tile; 13]; 4],
    },

    Tsumo {
        actor: u8,
        pai: Tile,
    },
    Dahai {
        actor: u8,
        pai: Tile,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Pon {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Daiminkan {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Kakan {
        actor: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Ankan {
        actor: u8,
        consumed: [Tile; 4],
    },
    Dora {
        dora_marker: Tile,
    },

    Reach {
        actor: u8,
    },
    ReachAccepted {
        actor: u8,
    },

    Hora {
        actor: u8,
        target: u8,
        #[serde(default)]
        deltas: Option<[i32; 4]>,
        #[serde(default)]
        ura_markers: Option<Vec<Tile>>,
    },
    Ryukyoku {
        #[serde(default)]
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

/// An extended version of `Event` which allows metadata recording.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventExt {
    #[serde(flatten)]
    pub event: Event,
    pub meta: Option<Metadata>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub q_values: Option<Vec<f32>>,
    pub mask_bits: Option<u64>,
    pub is_greedy: Option<bool>,
    pub batch_size: Option<usize>,
    pub eval_time_ns: Option<u64>,
    pub kan_select: Option<Box<Metadata>>,
    pub extra: Option<Value>,
}

impl Event {
    #[inline]
    pub const fn actor(&self) -> Option<u8> {
        match *self {
            Event::Tsumo { actor, .. }
            | Event::Dahai { actor, .. }
            | Event::Chi { actor, .. }
            | Event::Pon { actor, .. }
            | Event::Daiminkan { actor, .. }
            | Event::Kakan { actor, .. }
            | Event::Ankan { actor, .. }
            | Event::Reach { actor, .. }
            | Event::ReachAccepted { actor, .. }
            | Event::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }
}

impl EventExt {
    #[inline]
    pub const fn no_meta(event: Event) -> Self {
        Self { event, meta: None }
    }
}

impl From<Event> for EventExt {
    fn from(ev: Event) -> Self {
        Self::no_meta(ev)
    }
}
