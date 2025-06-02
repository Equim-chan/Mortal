use crate::tile::Tile;
use std::error::Error;
use std::fmt;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::{TryFromInto, serde_as, skip_serializing_none};

/// Describes an event in mjai format.
///
/// Mjai protocol was originally defined in
/// <https://gimite.net/pukiwiki/index.php?Mjai%20%E9%BA%BB%E9%9B%80AI%E5%AF%BE%E6%88%A6%E3%82%B5%E3%83%BC%E3%83%90>.
/// This implementation does not contain the full specs defined in the original
/// one, and it has some extensions added.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, PartialEq, Eq, Derivative, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    #[default]
    None,

    StartGame {
        #[serde(default)]
        names: [String; 4],

        /// Consists of (nonce, key).
        seed: Option<(u64, u64)>,
    },
    StartKyoku {
        bakaze: Tile,
        dora_marker: Tile,
        /// Counts from 1
        #[serde_as(deserialize_as = "TryFromInto<BoundedU8<1, 4>>")]
        kyoku: u8,
        honba: u8,
        kyotaku: u8,
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        oya: u8,
        scores: [i32; 4],
        tehais: [[Tile; 13]; 4],
    },

    Tsumo {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        pai: Tile,
    },
    Dahai {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        pai: Tile,
        tsumogiri: bool,
    },

    Chi {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Pon {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Daiminkan {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        target: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Kakan {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Ankan {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        consumed: [Tile; 4],
    },
    Dora {
        dora_marker: Tile,
    },

    Reach {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
    },
    ReachAccepted {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
    },

    Hora {
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        actor: u8,
        #[serde_as(deserialize_as = "TryFromInto<Actor>")]
        target: u8,

        deltas: Option<[i32; 4]>,
        ura_markers: Option<Vec<Tile>>,
    },
    Ryukyoku {
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

#[derive(Deserialize)]
struct BoundedU8<const MIN: u8, const MAX: u8>(u8);

type Actor = BoundedU8<0, 3>;

#[derive(Debug)]
pub struct OutOfBoundError(pub u8);

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
    pub shanten: Option<i8>,
    pub at_furiten: Option<bool>,
    pub kan_select: Option<Box<Metadata>>,
}

#[derive(Serialize, Deserialize)]
pub struct EventWithCanAct {
    #[serde(flatten)]
    pub event: Event,
    pub can_act: Option<bool>,
}

impl Event {
    #[inline]
    #[must_use]
    pub const fn actor(&self) -> Option<u8> {
        match *self {
            Self::Tsumo { actor, .. }
            | Self::Dahai { actor, .. }
            | Self::Chi { actor, .. }
            | Self::Pon { actor, .. }
            | Self::Daiminkan { actor, .. }
            | Self::Kakan { actor, .. }
            | Self::Ankan { actor, .. }
            | Self::Reach { actor, .. }
            | Self::ReachAccepted { actor, .. }
            | Self::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_in_game_announce(&self) -> bool {
        matches!(
            self,
            Self::ReachAccepted { .. } | Self::Dora { .. } | Self::Hora { .. }
        )
    }

    pub fn augment(&mut self) {
        const fn swap_tile(t: &mut Tile) {
            *t = t.augment();
        }

        match self {
            Self::StartKyoku {
                bakaze,
                dora_marker,
                tehais,
                ..
            } => {
                swap_tile(bakaze);
                swap_tile(dora_marker);
                tehais.iter_mut().flatten().for_each(swap_tile);
            }
            Self::Tsumo { pai, .. } | Self::Dahai { pai, .. } => swap_tile(pai),
            Self::Chi { pai, consumed, .. } | Self::Pon { pai, consumed, .. } => {
                swap_tile(pai);
                consumed.iter_mut().for_each(swap_tile);
            }
            Self::Daiminkan { pai, consumed, .. } | Self::Kakan { pai, consumed, .. } => {
                swap_tile(pai);
                consumed.iter_mut().for_each(swap_tile);
            }
            Self::Ankan { consumed, .. } => consumed.iter_mut().for_each(swap_tile),
            Self::Dora { dora_marker } => swap_tile(dora_marker),
            Self::Hora { ura_markers, .. } => ura_markers.iter_mut().flatten().for_each(swap_tile),
            _ => (),
        }
    }
}

impl<const MIN: u8, const MAX: u8> TryFrom<BoundedU8<MIN, MAX>> for u8 {
    type Error = OutOfBoundError;

    fn try_from(value: BoundedU8<MIN, MAX>) -> Result<Self, Self::Error> {
        if (MIN..=MAX).contains(&value.0) {
            Ok(value.0)
        } else {
            Err(OutOfBoundError(value.0))
        }
    }
}

impl fmt::Display for OutOfBoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "out-of-range number {}", self.0)
    }
}

impl Error for OutOfBoundError {}

impl EventExt {
    #[inline]
    #[must_use]
    pub const fn no_meta(event: Event) -> Self {
        Self { event, meta: None }
    }
}

impl From<Event> for EventExt {
    fn from(ev: Event) -> Self {
        Self::no_meta(ev)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json::{self as json, Map, Number, Value, json};

    #[test]
    fn json_consistency() {
        let lines = r#"
            {"type":"none"}
            {"type":"start_game","names":["Equim","Mortal","akochan","NoName"],"seed":[123,456]}
            {"type":"start_kyoku","bakaze":"E","dora_marker":"5s","kyoku":1,"honba":0,"kyotaku":0,"oya":0,"scores":[25000,25000,25000,25000],"tehais":[["N","3p","W","W","7m","N","S","C","7m","P","8p","2m","5m"],["7p","1p","2m","3m","4m","C","7s","7s","9s","9p","1m","C","1s"],["3s","E","5m","P","5m","F","7p","6m","5s","9p","1s","S","N"],["2p","4s","4p","E","5p","F","3p","1s","8p","6s","8s","7s","5p"]]}
            {"type":"tsumo","actor":0,"pai":"1m"}
            {"type":"dahai","actor":0,"pai":"2m","tsumogiri":true}
            {"type":"chi","actor":1,"target":0,"pai":"6s","consumed":["5sr","7s"]}
            {"type":"pon","actor":1,"target":0,"pai":"C","consumed":["C","C"]}
            {"type":"daiminkan","actor":2,"target":0,"pai":"5p","consumed":["5pr","5p","5p"]}
            {"type":"kakan","actor":3,"pai":"S","consumed":["S","S","S"]}
            {"type":"ankan","actor":0,"consumed":["9m","9m","9m","9m"]}
            {"type":"dora","dora_marker":"3s"}
            {"type":"reach","actor":1}
            {"type":"reach_accepted","actor":2}
            {"type":"hora","actor":3,"target":1,"deltas":[0,-8000,0,9000],"ura_markers":["4p"]}
            {"type":"hora","actor":3,"target":1}
            {"type":"ryukyoku","deltas":[0,1500,0,-1500]}
            {"type":"ryukyoku"}
            {"type":"end_kyoku"}
            {"type":"end_game"}
        "#.trim();

        let expected: Vec<Value> = lines.lines().map(|l| json::from_str(l).unwrap()).collect();
        let actual: Vec<Value> = lines
            .lines()
            .map(|l| {
                let event: Event = json::from_str(l).unwrap();
                json::to_value(event).unwrap()
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn bound_check() {
        let value = json! ({
            "type": "reach",
            "actor": 4,
        });
        json::from_value::<Event>(value).unwrap_err();

        let value = json! ({
            "type": "hora",
            "actor": 0,
            "target": 5,
        });
        json::from_value::<Event>(value).unwrap_err();

        let value = json!({
            "type": "start_kyoku",
            "bakaze": "E",
            "dora_marker": "5s",
            "kyoku": 1,
            "honba": 0,
            "kyotaku": 0,
            "oya": 0,
            "scores": [25000, 25000, 25000, 25000],
            "tehais": [
                ["N","3p","W","W","7m","N","S","C","7m","P","8p","2m","5m"],
                ["7p","1p","2m","3m","4m","C","7s","7s","9s","9p","1m","C","1s"],
                ["3s","E","5m","P","5m","F","7p","6m","5s","9p","1s","S","N"],
                ["2p","4s","4p","E","5p","F","3p","1s","8p","6s","8s","7s","5p"],
            ],
        });
        let obj: Map<String, Value> = json::from_value(value).unwrap();
        json::from_value::<Event>(Value::Object(obj.clone())).unwrap();

        let mut test_obj = obj.clone();
        test_obj["kyoku"] = Value::Number(Number::from(0));
        json::from_value::<Event>(Value::Object(test_obj)).unwrap_err();

        let mut test_obj = obj;
        test_obj["kyoku"] = Value::Number(Number::from(5));
        json::from_value::<Event>(Value::Object(test_obj)).unwrap_err();
    }
}
