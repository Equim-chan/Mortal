//! Rust port of EndlessCheng's Go port of 山岡忠夫's Java implementation of his
//! agari algorithm.
//!
//! Source:
//! * Go: <https://github.com/EndlessCheng/mahjong-helper/blob/master/util/agari.go>
//! * Java: <http://hp.vector.co.jp/authors/VA046927/mjscore/AgariIndex.java>
//! * Algorithm: <http://hp.vector.co.jp/authors/VA046927/mjscore/mjalgorism.html>

use super::point::Point;
use super::shanten;
use crate::tile::Tile;
use crate::{matches_tu8, must_tile, tu8};
use std::cmp::Ordering;
use std::iter;
use std::sync::LazyLock;

use boomphf::hashmap::BoomHashMap;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use tinyvec::ArrayVec;

const AGARI_TABLE_SIZE: usize = 9_362;

static AGARI_TABLE: LazyLock<BoomHashMap<u32, ArrayVec<[Div; 4]>>> = LazyLock::new(|| {
    let mut raw = GzDecoder::new(include_bytes!("data/agari.bin.gz").as_slice());

    let (keys, values): (Vec<_>, Vec<_>) = (0..AGARI_TABLE_SIZE)
        .map(|_| {
            let key = raw.read_u32::<LittleEndian>().unwrap();
            let v_size = raw.read_u8().unwrap();
            let value = (0..v_size)
                .map(|_| raw.read_u32::<LittleEndian>().unwrap())
                .map(Div::from)
                .collect();
            (key, value)
        })
        .unzip();

    if cfg!(test) {
        // Ensure there is no duplicated keys.
        let mut k = keys.clone();
        k.sort_unstable();
        k.dedup();
        assert_eq!(k.len(), keys.len());

        // Ensure there is no data left to read.
        raw.read_u8().unwrap_err();
    }

    BoomHashMap::new(keys, values)
});

#[derive(Debug, Default)]
struct Div {
    pair_idx: u8,
    kotsu_idxs: ArrayVec<[u8; 4]>,
    shuntsu_idxs: ArrayVec<[u8; 4]>,
    has_chitoi: bool,
    has_chuuren: bool,
    has_ittsuu: bool,
    has_ryanpeikou: bool,
    // CAUTION: it is sound but not complete, broken if there is any ankan
    has_ipeikou: bool,
}

#[derive(Debug, Clone, Copy, Eq)]
pub enum Agari {
    /// `fu` may be 0 if `han` is greater than 4.
    Normal {
        fu: u8,
        han: u8,
    },
    Yakuman(u8),
}

#[derive(Debug)]
pub struct AgariCalculator<'a> {
    /// Must include the winning tile (i.e. must be 3n+2)
    pub tehai: &'a [u8; 34],
    /// `self.chis.is_empty() && self.pons.is_empty() && self.minkans.is_empty()`
    pub is_menzen: bool,
    pub chis: &'a [u8],
    pub pons: &'a [u8],
    pub minkans: &'a [u8],
    pub ankans: &'a [u8],

    pub bakaze: u8,
    pub jikaze: u8,

    /// Must be deakaized
    pub winning_tile: u8,
    /// For consistency reasons, `is_ron` is only used to calculate fu and check
    /// ankou/ankan-related yakus like 三/四暗刻. It will not be used to
    /// determine 門前清自摸和.
    pub is_ron: bool,
}

struct DivWorker<'a> {
    sup: &'a AgariCalculator<'a>,
    tile14: &'a [u8; 14],
    div: &'a Div,
    pair_tile: u8,
    menzen_kotsu: ArrayVec<[u8; 4]>,
    menzen_shuntsu: ArrayVec<[u8; 4]>,

    /// Used in fu calc and sanankou condition, indicating whether or not the
    /// winning tile should build a minkou instead of shuntsu in an ambiguous
    /// pattern.
    ///
    /// The winning tile should try its best to fit into a shuntsu, because that
    /// always gives a higher score than using that winning tile to turn an
    /// existing ankou into a minkou, because a shuntsu can only add at most 2
    /// fu (penchan or kanchan) and does not bring extra yaku (except for pinfu,
    /// but since we have ankou it can never be pinfu), but an ankou adds at
    /// least 2 fu and can bring extra yakus like sanankou.
    ///
    /// An example of this is 45556 + 5, which could be either 456 + (55 + 5) or
    /// 555 + (46 + 5). `menzen_kotsu` will contain 555 while `menzen_shuntsu`
    /// will also contain 456, making it ambiguous whether the winning tile 5
    /// should be a part of either the minkou 55 + 5 or the shuntsu 46 + 5. In
    /// practice, the latter should be preferred because it preserves the ankou.
    /// A test case covers this.
    winning_tile_makes_minkou: bool,
}

impl From<u32> for Div {
    fn from(v: u32) -> Self {
        let pair_idx = ((v >> 6) & 0b1111) as u8;

        let kotsu_count = v & 0b111;
        let kotsu_idxs = (0..kotsu_count)
            .map(|i| ((v >> (10 + i * 4)) & 0b1111) as u8)
            .collect();

        let shuntsu_count = (v >> 3) & 0b111;
        let shuntsu_idxs = (kotsu_count..kotsu_count + shuntsu_count)
            .map(|i| ((v >> (10 + i * 4)) & 0b1111) as u8)
            .collect();

        let has_chitoi = (v >> 26) & 0b1 == 0b1;
        let has_chuuren = (v >> 27) & 0b1 == 0b1;
        let has_ittsuu = (v >> 28) & 0b1 == 0b1;
        let has_ryanpeikou = (v >> 29) & 0b1 == 0b1;
        let has_ipeikou = (v >> 30) & 0b1 == 0b1;

        Self {
            pair_idx,
            kotsu_idxs,
            shuntsu_idxs,
            has_chitoi,
            has_chuuren,
            has_ittsuu,
            has_ryanpeikou,
            has_ipeikou,
        }
    }
}

impl PartialEq for Agari {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Yakuman(l), Self::Yakuman(r)) => l == r,
            (Self::Normal { fu: lf, han: lh }, Self::Normal { fu: rf, han: rh }) => {
                lf == rf && lh == rh
            }
            _ => false,
        }
    }
}

impl PartialOrd for Agari {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Agari {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Yakuman(l), Self::Yakuman(r)) => l.cmp(r),
            (Self::Yakuman(_), Self::Normal { .. }) => Ordering::Greater,
            (Self::Normal { .. }, Self::Yakuman(..)) => Ordering::Less,
            (Self::Normal { fu: lf, han: lh }, Self::Normal { fu: rf, han: rh }) => {
                match lh.cmp(rh) {
                    Ordering::Equal => lf.cmp(rf),
                    v => v,
                }
            }
        }
    }
}

impl Agari {
    #[must_use]
    pub fn point(self, is_oya: bool) -> Point {
        match self {
            Self::Normal { fu, han } => Point::calc(is_oya, fu, han),
            Self::Yakuman(n) => Point::yakuman(is_oya, n as i32),
        }
    }
}

impl AgariCalculator<'_> {
    #[inline]
    #[must_use]
    pub fn has_yaku(&self) -> bool {
        self.search_yakus_impl(true).is_some()
    }

    #[inline]
    #[must_use]
    pub fn search_yakus(&self) -> Option<Agari> {
        self.search_yakus_impl(false)
    }

    /// `additional_hans` includes 門前清自摸和, (両)立直, 槍槓, 嶺上開花, 海底
    /// 摸月 and 河底撈魚. 天和 and 地和 are supposed to be checked somewhere
    /// else other than here.
    ///
    /// `None` is returned iff `!self.has_yaku() && additional_hans == 0` holds.
    ///
    /// This function is only supposed to be called by callers who have the
    /// knowledge of the ura doras.
    #[must_use]
    pub fn agari(&self, additional_hans: u8, doras: u8) -> Option<Agari> {
        if let Some(agari) = self.search_yakus() {
            Some(match agari {
                Agari::Normal { fu, han } => Agari::Normal {
                    fu,
                    han: han + additional_hans + doras,
                },
                _ => agari,
            })
        } else if additional_hans == 0 {
            None
        } else if additional_hans + doras >= 5 {
            Some(Agari::Normal {
                fu: 0,
                han: additional_hans + doras,
            })
        } else {
            let (tile14, key) = get_tile14_and_key(self.tehai);
            let divs = AGARI_TABLE.get(&key)?;

            let fu = divs
                .iter()
                .map(|div| DivWorker::new(self, &tile14, div))
                .map(|w| w.calc_fu(false))
                .max()?;
            Some(Agari::Normal {
                fu,
                han: additional_hans + doras,
            })
        }
    }

    fn search_yakus_impl(&self, return_if_any: bool) -> Option<Agari> {
        assert_eq!(
            self.is_menzen,
            self.chis.is_empty() && self.pons.is_empty() && self.minkans.is_empty(),
        );

        // Kokushi has a special pattern and cannot be combined with other
        // pattern-based yakus.
        if self.is_menzen && shanten::calc_kokushi(self.tehai) == -1 {
            // 国士無双
            return Some(Agari::Yakuman(1));
        }

        let (tile14, key) = get_tile14_and_key(self.tehai);
        let divs = AGARI_TABLE.get(&key)?;

        if return_if_any {
            // Benchmark result indicates it is too trivial to use rayon here.
            divs.iter()
                .map(|div| DivWorker::new(self, &tile14, div))
                .find_map(|w| w.search_yakus::<true>())
        } else {
            divs.iter()
                .map(|div| DivWorker::new(self, &tile14, div))
                .filter_map(|w| w.search_yakus::<false>())
                .max()
        }
    }
}

impl<'a> DivWorker<'a> {
    fn new(calc: &'a AgariCalculator<'a>, tile14: &'a [u8; 14], div: &'a Div) -> Self {
        let pair_tile = tile14[div.pair_idx as usize];
        let menzen_kotsu = div
            .kotsu_idxs
            .iter()
            .map(|&idx| tile14[idx as usize])
            .collect();
        let menzen_shuntsu = div
            .shuntsu_idxs
            .iter()
            .map(|&idx| tile14[idx as usize])
            .collect();

        let mut ret = Self {
            sup: calc,
            tile14,
            div,
            pair_tile,
            menzen_kotsu,
            menzen_shuntsu,
            winning_tile_makes_minkou: false,
        };
        ret.winning_tile_makes_minkou = ret.winning_tile_makes_minkou();
        ret
    }

    /// For init only.
    fn winning_tile_makes_minkou(&self) -> bool {
        if !self.sup.is_ron {
            // Tsumo agari, no way to make a minkou from the winning tile.
            return false;
        }
        if !self.menzen_kotsu.contains(&self.sup.winning_tile) {
            // No ankou that contains the winning tile, so no ambiguous pattern
            // at all.
            return false;
        }

        if self.sup.winning_tile >= 3 * 9 {
            // If the ron winning tile is jihai and makes a kotsu, then it must
            // be a minkou.
            return true;
        }
        let kind = self.sup.winning_tile / 9;
        let num = self.sup.winning_tile % 9;
        let low = kind * 9 + num.saturating_sub(2);
        let high = kind * 9 + num.min(6);
        // If there is a shuntsu that can cover the winning tile, then always
        // put the winning tile into that shuntsu.
        !(low..=high).any(|t| self.menzen_shuntsu.contains(&t))
    }

    /// The caller must assure `self.div.has_chitoi` holds.
    fn chitoi_pairs(&self) -> impl Iterator<Item = u8> + '_ {
        self.tile14.iter().take(7).copied()
    }

    fn all_kotsu_and_kantsu(&self) -> impl Iterator<Item = u8> + '_ {
        self.menzen_kotsu
            .iter()
            .chain(self.sup.pons)
            .chain(self.sup.minkans)
            .chain(self.sup.ankans)
            .copied()
    }

    fn all_shuntsu(&self) -> impl Iterator<Item = u8> + '_ {
        self.menzen_shuntsu.iter().chain(self.sup.chis).copied()
    }

    fn all_mentsu(&self) -> impl Iterator<Item = u8> + '_ {
        self.all_kotsu_and_kantsu().chain(self.all_shuntsu())
    }

    fn calc_fu(&self, has_pinfu: bool) -> u8 {
        if self.div.has_chitoi {
            return 25;
        }
        let mut fu = 20;

        fu += self
            .menzen_kotsu
            .iter()
            .map(|&t| {
                // `menzen_kotsu` are usually ankou, except when the winning
                // tile makes a minkou and the tile is the winning tile.
                let is_minkou = self.winning_tile_makes_minkou && t == self.sup.winning_tile;
                match (is_minkou, must_tile!(t).is_yaokyuu()) {
                    (false, true) => 8,
                    (false, false) | (true, true) => 4,
                    (true, false) => 2,
                }
            })
            .sum::<u8>();
        fu += self
            .sup
            .pons
            .iter()
            .map(|&t| if must_tile!(t).is_yaokyuu() { 4 } else { 2 })
            .sum::<u8>();
        fu += self
            .sup
            .ankans
            .iter()
            .map(|&t| if must_tile!(t).is_yaokyuu() { 32 } else { 16 })
            .sum::<u8>();
        fu += self
            .sup
            .minkans
            .iter()
            .map(|&t| if must_tile!(t).is_yaokyuu() { 16 } else { 8 })
            .sum::<u8>();

        if matches_tu8!(self.pair_tile, P | F | C) {
            fu += 2;
        } else {
            // As per [Tenhou's rule](https://tenhou.net/man/#RULE):
            //
            // > 連風牌は4符
            if self.pair_tile == self.sup.bakaze {
                fu += 2;
            }
            if self.pair_tile == self.sup.jikaze {
                fu += 2;
            }
        }

        if fu == 20 {
            return if !self.sup.is_menzen {
                30
            } else if has_pinfu {
                if self.sup.is_ron { 30 } else { 20 }
            } else if self.sup.is_ron {
                40
            } else {
                30
            };
        }

        if !self.sup.is_ron {
            fu += 2;
        } else if self.sup.is_menzen {
            fu += 10;
        }

        if !self.winning_tile_makes_minkou {
            if self.pair_tile == self.sup.winning_tile {
                // tanki wait
                fu += 2;
            } else {
                let is_kanchan_penchan = self.menzen_shuntsu.iter().any(|&s| {
                    s + 1 == self.sup.winning_tile
                        || s % 9 == 0 && s + 2 == self.sup.winning_tile
                        || s % 9 == 6 && s == self.sup.winning_tile
                });
                if is_kanchan_penchan {
                    fu += 2;
                }
            }
        }

        ((fu - 1) / 10 + 1) * 10
    }

    fn search_yakus<const RETURN_IF_ANY: bool>(&self) -> Option<Agari> {
        let mut han = 0;
        let mut yakuman = 0;

        let has_pinfu = self.menzen_shuntsu.len() == 4
            && !matches_tu8!(self.pair_tile, P | F | C)
            && self.pair_tile != self.sup.bakaze
            && self.pair_tile != self.sup.jikaze
            && self.menzen_shuntsu.iter().any(|&s| {
                let num = s % 9 + 1;
                num <= 6 && s == self.sup.winning_tile || num >= 2 && s + 2 == self.sup.winning_tile
            });

        macro_rules! make_return {
            () => {
                return if yakuman > 0 {
                    Some(Agari::Yakuman(yakuman))
                } else if han > 0 {
                    let fu = if RETURN_IF_ANY || han >= 5 {
                        0
                    } else {
                        self.calc_fu(has_pinfu)
                    };
                    Some(Agari::Normal { fu, han })
                } else {
                    None
                };
            };
        }
        macro_rules! check_early_return {
            ($($block:tt)*) => {{
                $($block)*;
                if RETURN_IF_ANY {
                    make_return!();
                }
            }};
        }

        if has_pinfu {
            // 平和
            check_early_return! { han += 1 };
        }
        if self.div.has_chitoi {
            // 七対子
            check_early_return! { han += 2 };
        }
        if self.div.has_ryanpeikou {
            // 二盃口
            check_early_return! { han += 3 };
        }
        if self.div.has_chuuren {
            // 九蓮宝燈
            check_early_return! { yakuman += 1 };
        }

        let has_tanyao = if self.div.has_chitoi {
            self.chitoi_pairs().all(|t| {
                let kind = t / 9;
                let num = t % 9;
                kind < 3 && num > 0 && num < 8
            })
        } else {
            self.all_shuntsu().all(|s| {
                let num = s % 9;
                num > 0 && num < 6
            }) && self
                .all_kotsu_and_kantsu()
                .chain(iter::once(self.pair_tile))
                .all(|k| {
                    let kind = k / 9;
                    let num = k % 9;
                    kind < 3 && num > 0 && num < 8
                })
        };
        if has_tanyao {
            // 断幺九
            check_early_return! { han += 1 };
        }

        let has_toitoi =
            !self.div.has_chitoi && self.menzen_shuntsu.is_empty() && self.sup.chis.is_empty();
        if has_toitoi {
            // 対々和
            check_early_return! { han += 2 };
        }

        let mut isou_kind = None;
        let mut has_jihai = false;
        let mut is_chinitsu_or_honitsu = true;
        let iter_fn = |&m: &u8| {
            let kind = m / 9;
            if kind >= 3 {
                has_jihai = true;
                return true;
            }
            if let Some(prev_kind) = isou_kind {
                if prev_kind != kind {
                    is_chinitsu_or_honitsu = false;
                    return false;
                }
            } else {
                isou_kind = Some(kind);
            }
            true
        };
        if self.div.has_chitoi {
            self.chitoi_pairs().take_while(iter_fn).for_each(drop);
        } else {
            self.all_mentsu()
                .chain(iter::once(self.pair_tile))
                .take_while(iter_fn)
                .for_each(drop);
        }
        if isou_kind.is_none() {
            // 字一色
            check_early_return! { yakuman += 1 };
        } else if is_chinitsu_or_honitsu {
            // 混一色, 清一色
            let n = if has_jihai { 2 } else { 5 } + self.sup.is_menzen as u8;
            check_early_return! { han += n };
        }

        if !self.div.has_chitoi {
            // 一盃口
            if self.div.has_ipeikou {
                check_early_return! { han += 1 };
            } else if !self.sup.ankans.is_empty()
                && self.sup.is_menzen
                && self.menzen_shuntsu.len() >= 2
            {
                let mut shuntsu_marks = [0_u8; 3];
                let has_ipeikou = self.menzen_shuntsu.iter().any(|&t| {
                    let kind = t as usize / 9;
                    let num = t % 9;
                    let mark = &mut shuntsu_marks[kind];
                    if (*mark >> num) & 0b1 == 0b1 {
                        true
                    } else {
                        *mark |= 0b1 << num;
                        false
                    }
                });
                if has_ipeikou {
                    check_early_return! { han += 1 };
                }
            }

            // 一気通貫
            if self.sup.is_menzen && self.div.has_ittsuu {
                check_early_return! { han += 2 };
            } else if self.sup.chis.is_empty() && self.div.has_ittsuu {
                check_early_return! { han += 1 };
            } else if self.menzen_shuntsu.len() + self.sup.chis.len() >= 3 {
                let mut kinds = [0; 3];
                for s in self.all_shuntsu() {
                    let kind = s as usize / 9;
                    let num = s % 9;
                    match num {
                        0 => kinds[kind] |= 0b001,
                        3 => kinds[kind] |= 0b010,
                        6 => kinds[kind] |= 0b100,
                        _ => (),
                    };
                }
                if kinds.contains(&0b111) {
                    check_early_return! { han += 1 };
                }
            }

            let mut s_counter = [0; 9];
            for s in self.all_shuntsu() {
                let kind = s / 9;
                let num = s % 9;
                s_counter[num as usize] |= 0b1 << kind;
            }
            if s_counter.contains(&0b111) {
                // 三色同順
                let n = if self.sup.is_menzen { 2 } else { 1 };
                check_early_return! { han += n };
            } else {
                let mut k_counter = [0; 9];
                for k in self.all_kotsu_and_kantsu() {
                    let kind = k / 9;
                    if kind < 3 {
                        let num = k % 9;
                        k_counter[num as usize] |= 1 << kind;
                    }
                }
                if k_counter.contains(&0b111) {
                    // 三色同刻
                    check_early_return! { han += 2 };
                }
            }

            let ankous_count = self.sup.ankans.len() + self.menzen_kotsu.len()
                - self.winning_tile_makes_minkou as usize;
            match ankous_count {
                // 四暗刻
                4 => check_early_return! { yakuman += 1 },
                // 三暗刻
                3 => check_early_return! { han += 2 },
                _ => (),
            };

            let kans_count = self.sup.ankans.len() + self.sup.minkans.len();
            match kans_count {
                // 四槓子
                4 => check_early_return! { yakuman += 1 },
                // 三槓子
                3 => check_early_return! { han += 2 },
                _ => (),
            };

            let has_ryuisou = self
                .all_kotsu_and_kantsu()
                .chain(iter::once(self.pair_tile))
                .all(|k| matches_tu8!(k, 2s | 3s | 4s | 6s | 8s | F))
                && self.all_shuntsu().all(|s| s == tu8!(2s)); // only 234s is possible for shuntsu in ryuisou
            if has_ryuisou {
                // 緑一色
                check_early_return! { yakuman += 1 };
            }

            if !has_tanyao {
                // 役牌 + 大小三元四喜
                let mut has_jihai = [false; 7];
                for k in self.all_kotsu_and_kantsu() {
                    if k >= 3 * 9 {
                        has_jihai[k as usize - 3 * 9] = true;
                    }
                }
                if has_jihai[self.sup.bakaze as usize - 3 * 9] {
                    // 役牌:門風牌
                    check_early_return! { han += 1 };
                }
                if has_jihai[self.sup.jikaze as usize - 3 * 9] {
                    // 役牌:場風牌
                    check_early_return! { han += 1 };
                }

                let saneins = (4..7).filter(|&i| has_jihai[i]).count() as u8;
                if saneins > 0 {
                    // 役牌:三元牌
                    check_early_return! { han += saneins };
                    if saneins == 3 {
                        // 大三元
                        check_early_return! { yakuman += 1 };
                    } else if saneins == 2 && matches_tu8!(self.pair_tile, P | F | C) {
                        // 小三元
                        check_early_return! { han += 2 };
                    }
                }

                let winds = (0..4).filter(|&i| has_jihai[i]).count();
                #[allow(clippy::if_same_then_else)]
                if winds == 4 {
                    // 大四喜
                    check_early_return! { yakuman += 1 };
                } else if winds == 3 && matches_tu8!(self.pair_tile, E | S | W | N) {
                    // 小四喜
                    check_early_return! { yakuman += 1 };
                }
            }
        }

        if !has_tanyao {
            let mut has_jihai = false;
            let is_yaokyuu = |k| {
                let kind = k / 9;
                if kind >= 3 {
                    has_jihai = true;
                    true
                } else {
                    let num = k % 9;
                    num == 0 || num == 8
                }
            };
            let is_junchan_or_chanta_or_chinroutou_or_honroutou = if self.div.has_chitoi {
                self.chitoi_pairs().all(is_yaokyuu)
            } else {
                self.all_kotsu_and_kantsu()
                    .chain(iter::once(self.pair_tile))
                    .all(is_yaokyuu)
            };
            if is_junchan_or_chanta_or_chinroutou_or_honroutou {
                if self.div.has_chitoi || has_toitoi {
                    if has_jihai {
                        // 混老頭
                        check_early_return! { han += 2 };
                    } else {
                        // 清老頭
                        check_early_return! { yakuman += 1 };
                    }
                } else {
                    let is_junchan_or_chanta = self.all_shuntsu().all(|s| {
                        let num = s % 9;
                        num == 0 || num == 6
                    });
                    if is_junchan_or_chanta {
                        // 混全帯幺九, 純全帯幺九
                        let n = if has_jihai { 1 } else { 2 } + self.sup.is_menzen as u8;
                        check_early_return! { han += n };
                    }
                }
            }
        }

        make_return!();
    }
}

pub fn ensure_init() {
    assert_eq!(AGARI_TABLE.len(), AGARI_TABLE_SIZE);
}

fn get_tile14_and_key(tiles: &[u8; 34]) -> ([u8; 14], u32) {
    let mut tile14 = [0; 14];
    let mut tile14_iter = tile14.iter_mut();
    let mut key = 0;

    let mut bit_idx = -1;
    let mut prev_in_hand = None;
    for (kind, chunk) in tiles.chunks_exact(9).enumerate() {
        for (num, c) in chunk.iter().copied().enumerate() {
            if c > 0 {
                prev_in_hand = Some(());
                *tile14_iter.next().unwrap() = (kind * 9 + num) as u8;
                bit_idx += 1;

                match c {
                    2 => {
                        key |= 0b11 << bit_idx;
                        bit_idx += 2;
                    }
                    3 => {
                        key |= 0b1111 << bit_idx;
                        bit_idx += 4;
                    }
                    4 => {
                        key |= 0b11_1111 << bit_idx;
                        bit_idx += 6;
                    }
                    // 1
                    _ => (),
                }
            } else if prev_in_hand.take().is_some() {
                key |= 0b1 << bit_idx;
                bit_idx += 1;
            }
        }
        if prev_in_hand.take().is_some() {
            key |= 0b1 << bit_idx;
            bit_idx += 1;
        }
    }

    tiles
        .iter()
        .enumerate()
        .skip(3 * 9)
        .filter(|&(_, &c)| c > 0)
        .for_each(|(tile_id, &c)| {
            *tile14_iter.next().unwrap() = tile_id as u8;
            bit_idx += 1;

            match c {
                2 => {
                    key |= 0b11 << bit_idx;
                    bit_idx += 2;
                }
                3 => {
                    key |= 0b1111 << bit_idx;
                    bit_idx += 4;
                }
                4 => {
                    key |= 0b11_1111 << bit_idx;
                    bit_idx += 6;
                }
                // 1
                _ => (),
            }
            key |= 0b1 << bit_idx;
            bit_idx += 1;
        });

    (tile14, key)
}

/// `tehai` must already contain `tile`. `true` is returned if making an ankan
/// with the tile is legal under the riichi'd `tehai`.
///
/// If `strict` is `false`, it is the same as [Tenhou's
/// rule](https://tenhou.net/man/#RULE):
///
/// > リーチ後の暗槓は待ちが変わらない場合のみ。送り槓不可、牌姿や役の増減は不
/// > 問。
///
/// If `strict` is `true`, it will also check the shape of tenpai and agari, but
/// will not check yaku anyways.
///
/// The behavior is undefined if `tehai` is not tenpai.
#[must_use]
pub fn check_ankan_after_riichi(tehai: &[u8; 34], len_div3: u8, tile: Tile, strict: bool) -> bool {
    let tile_id = tile.deaka().as_usize();
    if tehai[tile_id] != 4 {
        return false;
    }

    if tile_id >= 3 * 9 {
        return true;
    }

    let mut tehai_before_tsumo = *tehai;
    tehai_before_tsumo[tile_id] -= 1;

    (0..34)
        .filter(|&t| {
            if tehai_before_tsumo[t] == 4 {
                return false;
            }
            // Get all waits of the original hand
            let mut tmp = tehai_before_tsumo;
            tmp[t] += 1;
            shanten::calc_all(&tmp, len_div3) == -1
        })
        .all(|wait| {
            // Cannot kan a waited tile
            if wait == tile_id {
                return false;
            }

            // Test if the hand after ankan can also win with the wait tile
            let mut tehai_after = *tehai;
            tehai_after[tile_id] = 0;
            tehai_after[wait] += 1;
            let (_, key) = get_tile14_and_key(&tehai_after);
            let Some(divs_after) = AGARI_TABLE.get(&key) else {
                // The wait tile set will get smaller after kan.
                return false;
            };

            if strict {
                // Compare if the number of hand divisions are equal before and
                // after ankan, which indicates the shapes of tenpai and agari
                // will not change after ankan. This is implemented by inserting
                // the waited tile to both of them.
                let mut tehai_before = tehai_before_tsumo;
                tehai_before[wait] += 1;
                let (_, key) = get_tile14_and_key(&tehai_before);
                let divs_before = AGARI_TABLE
                    .get(&key)
                    .expect("invalid riichi detected when testing ankan after riichi");

                if divs_after.len() != divs_before.len() {
                    return false;
                }
            }

            true
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hand::hand;

    #[test]
    fn ankan_after_riichi() {
        let test_one = |tehai_str, tile_str: &str, len_div3, strict, expected| {
            let mut tehai = hand(tehai_str).unwrap();
            let tile: Tile = tile_str.parse().unwrap();
            tehai[tile.as_usize()] += 1;
            assert_eq!(
                check_ankan_after_riichi(&tehai, len_div3, tile, strict),
                expected,
                "failed for {tehai_str} + {tile_str}, expected {expected}",
            );
        };

        // Always positive
        test_one("12345m 567s 11222z", "S", 4, true, true);
        test_one("12345m 444567s 11z", "4s", 4, true, true);
        test_one("22m 11112356p 444s", "4s", 4, true, true);

        // Always negative
        test_one("123456m 4445s 111z", "4s", 4, true, false);
        test_one("123456m 4445s 111z", "4s", 4, false, false);

        // Shape of tenpai changes
        test_one("1113444p 222z", "1p", 3, true, false);
        test_one("1113444p 222z", "1p", 3, false, true);
        test_one("1113444p 222z", "4p", 3, true, false);
        test_one("1113444p 222z", "S", 3, true, true);

        // Shape of agari changes
        test_one("23m 999p 33345666s", "3s", 4, true, false);
        test_one("23m 999p 33345666s", "6s", 4, true, false);
        test_one("23m 999p 33345666s", "6s", 4, false, true);
        test_one("23m 999p 33345666s", "9p", 4, true, true);

        // The 1m kan will make chuuren gone, but in this impl we don't take
        // yaku into account.
        test_one("1113445678999m", "1m", 4, true, true);
        test_one("1113445678999m", "9m", 4, true, false);
    }

    #[test]
    fn agari_calc() {
        let tehai = hand("2234455m 234p 234s 3m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(S),
            winning_tile: tu8!(3m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        assert_eq!(yaku, Agari::Normal { fu: 40, han: 4 });

        let tehai = hand("12334m 345p 22s 777z 2m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(3m),
            is_ron: false,
        };
        let points = calc.agari(2, 0).unwrap().point(true);
        // 立直, 門前清自摸和
        assert_eq!(
            points,
            Point {
                ron: 7700,
                tsumo_oya: 0,
                tsumo_ko: 2600
            }
        );

        let tehai = hand("2255m 445p 667788s 5p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(S),
            winning_tile: tu8!(5p),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        assert_eq!(yaku, Agari::Normal { fu: 25, han: 3 });
        assert_eq!(yaku.point(false).ron, 3200);

        let tehai = hand("22334m 33p 4m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &tu8![2s, 2s],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(S),
            winning_tile: tu8!(4m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        assert_eq!(yaku, Agari::Normal { fu: 30, han: 1 });

        let tehai = hand("223344p 667788s 3m 3m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(S),
            jikaze: tu8!(N),
            winning_tile: tu8!(3m),
            is_ron: false,
        };
        let yaku = calc.search_yakus().unwrap();
        assert_eq!(yaku, Agari::Normal { fu: 30, han: 4 });

        let tehai = hand("234678m 1123488p 8p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(8p),
            is_ron: true,
        };
        assert_eq!(calc.search_yakus(), None);

        let tehai = hand("223344999m 1188p 8p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(8p),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 一盃口 (without ankan)
        assert_eq!(yaku, Agari::Normal { fu: 40, han: 1 });

        let tehai = hand("223344m 1188p 8p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &tu8![9m,],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(8p),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 一盃口 (with ankan)
        assert_eq!(yaku, Agari::Normal { fu: 70, han: 1 });

        let tehai = hand("55566677m 11p 7m").unwrap();
        let mut calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &tu8![9s,],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(7m),
            is_ron: false,
        };
        let yaku = calc.search_yakus().unwrap();
        // 四暗刻
        assert_eq!(yaku, Agari::Yakuman(1));

        calc.is_ron = true;
        let yaku = calc.search_yakus().unwrap();
        // 三暗刻, 対々和
        assert_eq!(yaku, Agari::Normal { fu: 80, han: 4 });

        let tehai = hand("666677778888m 99p").unwrap();
        let mut calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(8m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 平和, 二盃口
        assert_eq!(yaku, Agari::Normal { fu: 30, han: 4 });

        calc.winning_tile = tu8!(7m);
        let yaku = calc.search_yakus().unwrap();
        // 二盃口
        assert_eq!(yaku, Agari::Normal { fu: 40, han: 3 });

        let tehai = hand("12345678m 11p 9m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &tu8![9p,],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(9m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 一気通貫
        assert_eq!(yaku, Agari::Normal { fu: 70, han: 2 });

        let tehai = hand("12345678m 11p 9m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &[],
            pons: &tu8![9p,],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(9m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 一気通貫
        assert_eq!(yaku, Agari::Normal { fu: 30, han: 1 });

        let tehai = hand("111222333m 67p 88s 8p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(8p),
            is_ron: false,
        };
        let yaku = calc.search_yakus().unwrap();
        // 門前清自摸和 is not accounted.
        assert_eq!(yaku, Agari::Normal { fu: 40, han: 2 });

        let tehai = hand("1112223334447z 7z").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(C),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        assert_eq!(yaku, Agari::Yakuman(3));

        let tehai = hand("1m 789p 789s 1m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &tu8![7m, 1s],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(1m),
            is_ron: false,
        };
        let yaku = calc.search_yakus().unwrap();
        // 純全, 三色
        assert_eq!(yaku, Agari::Normal { fu: 30, han: 3 });

        let tehai = hand("111444m 45556s 22z 5s").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(S),
            jikaze: tu8!(S),
            winning_tile: tu8!(5s),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 三暗刻 (5s is ankou)
        // 20 + menzenron(10) + kanchan(2) + bakaze(2) + jikaze(2) + 1m(8) + 4m(4)
        // + 5s(4) + = 52
        assert_eq!(yaku, Agari::Normal { fu: 60, han: 2 });

        let tehai = hand("999s 1777z 1z").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &tu8![1p,],
            pons: &tu8![N,],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(S),
            jikaze: tu8!(S),
            winning_tile: tu8!(E),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 混全帯幺九, 役牌*1
        // 20 + tanki(2) + 9s(8) + 7z(8) + 4z(4) = 42
        assert_eq!(yaku, Agari::Normal { fu: 50, han: 2 });

        let tehai = hand("1119m 9m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &[],
            pons: &tu8![S, C],
            minkans: &[],
            ankans: &tu8![N,],
            bakaze: tu8!(S),
            jikaze: tu8!(N),
            winning_tile: tu8!(9m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 混一色, 混老頭, 役牌*3, 対々和
        assert!(matches!(yaku, Agari::Normal { han: 9, .. }));
        let (tile14, key) = get_tile14_and_key(&tehai);
        let divs = AGARI_TABLE.get(&key).unwrap();
        let fu = divs
            .iter()
            .map(|div| DivWorker::new(&calc, &tile14, div))
            .map(|w| w.calc_fu(false))
            .max()
            .unwrap();
        // 20 + tanki(2) + 1m(8) + 2z(4) + 7z(4) + 4z(32) = 70
        assert_eq!(fu, 70);

        // This shape is called 八蓮宝燈, waiting on 12456789
        let tehai = hand("1233334567888m 9m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(9m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 清一色, 一気通貫
        assert!(matches!(yaku, Agari::Normal { han: 8, .. }));

        // This shape is called 七蓮宝燈, waiting on 1235789
        let tehai = hand("2344445666678p 5p").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(5p),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 清一色, 断么九
        assert!(matches!(yaku, Agari::Normal { han: 7, .. }));

        // Waits on 13467s
        let tehai = hand("2223445566s 1s").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: false,
            chis: &tu8![7s,],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(1s),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 清一色, 一気通貫
        assert!(matches!(yaku, Agari::Normal { han: 6, .. }));

        // Waits on 14m
        let tehai = hand("1123444m 111p 111s 1m").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(E),
            jikaze: tu8!(E),
            winning_tile: tu8!(1m),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 清一色, 一気通貫
        assert_eq!(yaku, Agari::Normal { fu: 60, han: 2 });

        let tehai = hand("111s 2225556677z 7z").unwrap();
        let calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: true,
            chis: &[],
            pons: &[],
            minkans: &[],
            ankans: &[],
            bakaze: tu8!(S),
            jikaze: tu8!(S),
            winning_tile: tu8!(C),
            is_ron: true,
        };
        let yaku = calc.search_yakus().unwrap();
        // 三暗刻, 対々和, 混一色, 混老頭, 小三元, double 南, 白, 中
        assert!(matches!(yaku, Agari::Normal { han: 15, .. }));
    }
}
