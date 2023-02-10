use super::action::ActionCandidate;
use super::item::{ChiPon, KawaItem, Sutehai};
use super::PlayerState;
use crate::algo::agari::{self, AgariCalculator};
use crate::algo::shanten;
use crate::mjai::Event;
use crate::rankings::Rankings;
use crate::tile::Tile;
use crate::{must_tile, tu8, tuz};
use std::cmp::Ordering;
use std::mem;

use tinyvec::array_vec;

#[derive(Clone, Copy)]
pub(super) enum MoveType {
    Tsumo,
    Discard,
    FuuroConsume,
}

impl PlayerState {
    #[inline]
    pub fn update(&mut self, event: &Event) -> ActionCandidate {
        self.update_with_skip(event, false)
    }

    pub fn update_with_skip(&mut self, event: &Event, skip_on_announce: bool) -> ActionCandidate {
        if !skip_on_announce || !event.is_in_game_announce() {
            self.last_cans = ActionCandidate {
                target_actor: event.actor().unwrap_or(self.player_id),
                ..Default::default()
            };

            if self.to_mark_same_cycle_furiten.take().is_some() {
                self.at_furiten = true;
            }
            if self.chankan_chance.take().is_some() {
                self.at_ippatsu = false;
            }

            self.ankan_candidates.clear();
            self.kakan_candidates.clear();
        }

        match *event {
            Event::StartKyoku {
                bakaze,
                dora_marker,
                kyoku,
                honba,
                kyotaku,
                oya,
                scores,
                tehais,
            } => {
                self.tehai.fill(0);
                self.waits.fill(false);
                self.dora_factor.fill(0);
                self.tiles_seen.fill(0);
                self.keep_shanten_discards.fill(false);
                self.next_shanten_discards.fill(false);
                self.forbidden_tiles.fill(false);
                self.discarded_tiles.fill(false);

                self.bakaze = bakaze;
                self.honba = honba;
                self.kyotaku = kyotaku;
                self.oya = self.rel(oya) as u8;
                self.jikaze = must_tile!(tu8!(E) + (4 - self.oya) % 4);
                self.kyoku = kyoku - 1;
                self.is_all_last = match self.bakaze.as_u8() {
                    tu8!(E) => false,
                    tu8!(S) => self.kyoku == 3,
                    _ => true,
                };

                self.scores = scores;
                self.scores.rotate_left(self.player_id as usize);

                self.dora_indicators.clear();
                self.doras_owned.fill(0);
                self.doras_seen = 0;
                self.akas_in_hand.fill(false);

                self.ankan_candidates.clear();
                self.kakan_candidates.clear();
                self.chankan_chance = None;

                self.at_ippatsu = false;
                self.at_rinshan = false;
                self.at_furiten = false;
                self.to_mark_same_cycle_furiten = None;

                self.is_menzen = true;
                self.can_w_riichi = true;
                self.is_w_riichi = false;
                self.chis.clear();
                self.pons.clear();
                self.minkans.clear();
                self.ankans.clear();

                self.kans_on_board = 0;
                self.tehai_len_div3 = 4;
                self.has_next_shanten_discard = false;
                self.tiles_left = 70;
                self.at_turn = 0;

                self.kawa.iter_mut().for_each(|k| k.clear());
                self.last_tedashis.fill(None);
                self.kawa_overview.iter_mut().for_each(|k| k.clear());
                self.fuuro_overview.iter_mut().for_each(|k| k.clear());
                self.ankan_overview.iter_mut().for_each(|k| k.clear());
                self.intermediate_kan.clear();
                self.intermediate_chi_pon = None;

                self.riichi_declared.fill(false);
                self.riichi_accepted.fill(false);
                self.riichi_sutehais.fill(None);

                self.last_self_tsumo = None;
                self.last_kawa_tile = None;

                // The updates must be in order and must be placed after all the
                // resets above.
                self.update_rank();
                self.add_dora_indicator(dora_marker);
                for &t in &tehais[self.player_id as usize] {
                    self.witness_tile(t);
                    self.move_tile(t, MoveType::Tsumo);
                }
                self.update_shanten();
                self.update_waits_and_furiten();
                self.pad_kawa_at_start();
            }

            Event::Tsumo { actor, pai } => {
                self.tiles_left -= 1;
                if actor != self.player_id {
                    return self.last_cans;
                }
                self.at_turn += 1;

                self.last_cans.can_discard = true;
                self.last_self_tsumo = Some(pai);
                self.witness_tile(pai);
                self.move_tile(pai, MoveType::Tsumo);

                if self.can_w_riichi {
                    self.last_cans.can_ryukyoku = self.yaokyuu_kind_count() >= 9;
                }

                if !self.riichi_accepted[0] {
                    // Does not update shanten
                    self.update_shanten_discards();
                }

                if self.waits[pai.deaka().as_usize()] {
                    if self.is_menzen // 門前清自摸和
                        || /* 立直 */ self.riichi_accepted[0]
                        || /* 海底摸月 */ self.tiles_left == 0
                        || /* 嶺上開花 */ self.at_rinshan
                        || /* 天地和 */ self.can_w_riichi
                    {
                        self.last_cans.can_tsumo_agari = true;
                    } else {
                        let agari_calc = AgariCalculator {
                            tehai: &self.tehai,
                            is_menzen: self.is_menzen,
                            chis: &self.chis,
                            pons: &self.pons,
                            minkans: &self.minkans,
                            ankans: &self.ankans,
                            bakaze: self.bakaze.as_u8(),
                            jikaze: self.jikaze.as_u8(),
                            winning_tile: pai.deaka().as_u8(),
                            is_ron: false,
                        };
                        self.last_cans.can_tsumo_agari = agari_calc.has_yaku();
                    }
                }

                // haitei tile cannot be used to kakan or ankan
                if self.tiles_left == 0 {
                    return self.last_cans;
                }

                if self.riichi_accepted[0] {
                    if self.kans_on_board < 4 {
                        // Using Tenhou rule here.
                        self.last_cans.can_ankan = agari::check_ankan_after_riichi(
                            &self.tehai,
                            self.tehai_len_div3,
                            pai,
                            false,
                        );
                        if self.last_cans.can_ankan {
                            self.ankan_candidates.push(pai.deaka());
                        }
                    }
                    return self.last_cans;
                }

                if self.kans_on_board < 4 {
                    self.tehai
                        .iter()
                        .enumerate()
                        .filter(|(_, &count)| count > 0)
                        .for_each(|(tid, &count)| {
                            let tile = must_tile!(tid);
                            if count == 4 {
                                self.last_cans.can_ankan = true;
                                self.ankan_candidates.push(tile);
                            } else if self.pons.contains(&(tid as u8)) {
                                self.last_cans.can_kakan = true;
                                self.kakan_candidates.push(tile);
                            }
                        });
                }

                self.last_cans.can_riichi = self.is_menzen
                    && self.tiles_left >= 4
                    && self.scores[0] >= 1000
                    && (self.shanten == 0 || self.shanten == 1 && self.has_next_shanten_discard);
            }

            Event::Dahai {
                actor,
                pai,
                tsumogiri,
            } => {
                let actor_rel = self.rel(actor);

                let is_riichi = self.riichi_declared[actor_rel] && !self.riichi_accepted[actor_rel];
                let sutehai = Sutehai {
                    tile: pai,
                    is_dora: self.dora_factor[pai.deaka().as_usize()] > 0,
                    is_tedashi: !tsumogiri,
                    is_riichi,
                };
                let kawa_item = KawaItem {
                    kan: mem::take(&mut self.intermediate_kan),
                    chi_pon: self.intermediate_chi_pon.take(),
                    sutehai,
                };
                self.kawa[actor_rel].push(Some(kawa_item));
                self.kawa_overview[actor_rel].push(pai);
                self.last_kawa_tile = Some(pai);

                if !tsumogiri {
                    self.last_tedashis[actor_rel] = Some(sutehai);
                }
                if is_riichi {
                    self.riichi_sutehais[actor_rel] = Some(sutehai);
                }

                if actor_rel == 0 {
                    self.forbidden_tiles.fill(false);
                    self.move_tile(pai, MoveType::Discard);

                    self.at_rinshan = false;
                    self.at_ippatsu = false;
                    self.can_w_riichi = false;

                    self.discarded_tiles[pai.deaka().as_usize()] = true;

                    // Furiten state will be permanent once riichi is accepted,
                    // and of course, the shanten number will be frozen as well,
                    // so the calculations are skipped here.
                    if !self.riichi_accepted[0] {
                        if self.next_shanten_discards[pai.deaka().as_usize()] {
                            self.shanten -= 1;
                        } else if !self.keep_shanten_discards[pai.deaka().as_usize()] {
                            self.update_shanten();
                        }
                        // Update is here because `self.tiles_seen` has
                        // changed so waits may have been changed, also the
                        // discarded `pai` might be a winning tile (tsumo agari
                        // minogashi) thus furiten status needs to update.
                        self.update_waits_and_furiten();
                    } else if !self.at_furiten && self.waits[pai.deaka().as_usize()] {
                        // Riichi furiten
                        self.at_furiten = true;
                    }

                    return self.last_cans;
                }
                self.witness_tile(pai);

                if !self.at_furiten && self.waits[pai.deaka().as_usize()] {
                    if self.riichi_accepted[0] || self.tiles_left == 0 {
                        // 立直 or 河底撈魚
                        self.last_cans.can_ron_agari = true;
                    } else {
                        let mut tehai_with_winning_tile = self.tehai;
                        tehai_with_winning_tile[pai.deaka().as_usize()] += 1;

                        let agari_calc = AgariCalculator {
                            tehai: &tehai_with_winning_tile,
                            is_menzen: self.is_menzen,
                            chis: &self.chis,
                            pons: &self.pons,
                            minkans: &self.minkans,
                            ankans: &self.ankans,
                            bakaze: self.bakaze.as_u8(),
                            jikaze: self.jikaze.as_u8(),
                            winning_tile: pai.deaka().as_u8(),
                            is_ron: true,
                        };
                        self.last_cans.can_ron_agari = agari_calc.has_yaku();
                    }

                    // Track same-cycle furiten
                    if self.last_cans.can_ron_agari {
                        // The hand has a yaku (can ron), but if the player does
                        // not ron at the next event, it will turn into a
                        // same-cycle furiten.
                        //
                        // Mark as furiten at the next event. We do not set
                        // `self.at_furiten = true` immediately because that
                        // would affect a likely feature encoding call right
                        // after this Dahai event.
                        self.to_mark_same_cycle_furiten = Some(());
                    } else {
                        // The hand doesn't have yaku. This is a no-yaku
                        // furiten.
                        //
                        // Mark as furiten immediately, following the behavior
                        // of Tenhou's furiten display.
                        self.at_furiten = true;
                    }
                }

                if self.riichi_accepted[0] || self.tiles_left == 0 {
                    return self.last_cans;
                }

                if actor_rel == 3 && !pai.is_jihai() && self.tehai_len_div3 > 0 {
                    self.set_can_chi_from_tile(pai);
                }
                self.last_cans.can_pon = self.tehai[pai.deaka().as_usize()] >= 2;
                self.last_cans.can_daiminkan =
                    self.kans_on_board < 4 && self.tehai[pai.deaka().as_usize()] == 3;
            }

            Event::Chi {
                actor,
                consumed,
                pai,
                ..
            } => {
                let actor_rel = self.rel(actor);
                let mut result = array_vec!();
                result.extend_from_slice(&consumed);
                result.push(pai);
                self.fuuro_overview[actor_rel].push(result);
                self.intermediate_chi_pon = Some(ChiPon {
                    consumed,
                    target_tile: pai,
                });

                if actor_rel != 0 {
                    consumed.into_iter().for_each(|t| self.witness_tile(t));
                    result
                        .into_iter()
                        .for_each(|t| self.update_doras_owned(actor_rel, t));
                    self.can_w_riichi = false;
                    self.at_ippatsu = false;
                    return self.last_cans;
                }

                self.last_cans.can_discard = true;
                self.is_menzen = false;
                self.tehai_len_div3 -= 1;
                // Marked explicitly as `None` to let `Agent` impls set
                // `tsumogiri` to false in the Dahai after Chi
                self.last_self_tsumo = None;

                self.update_doras_owned(0, pai);
                consumed
                    .into_iter()
                    .for_each(|t| self.move_tile(t, MoveType::FuuroConsume));

                let a = consumed[0].deaka().as_usize();
                let b = consumed[1].deaka().as_usize();
                let min = a.min(b);
                let max = a.max(b);
                let deaka_tile_id = pai.deaka().as_usize();
                self.chis.push(min.min(deaka_tile_id) as u8);

                // Forbid 喰い替え
                if self.tehai[deaka_tile_id] > 0 {
                    self.forbidden_tiles[deaka_tile_id] = true;
                }
                if deaka_tile_id < min {
                    if max % 9 < 8 {
                        // Like 56s chi 4s, then 7s is not allowed to discard
                        let bigger = max + 1;
                        if self.tehai[bigger] > 0 {
                            self.forbidden_tiles[bigger] = true;
                        }
                    }
                } else if deaka_tile_id > max && min % 9 > 0 {
                    // Like 56s chi 7s, then 4s is not allowed to discard
                    let smaller = min - 1;
                    if self.tehai[smaller] > 0 {
                        self.forbidden_tiles[smaller] = true;
                    }
                }

                // NOTES: this is 3n+2
                // The shanten can change after chi, for example 1235578 chi 4.
                self.update_shanten();
                self.update_shanten_discards();
            }

            Event::Pon {
                actor,
                target,
                consumed,
                pai,
            } => {
                let actor_rel = self.rel(actor);
                let mut result = array_vec!();
                result.extend_from_slice(&consumed);
                result.push(pai);
                self.fuuro_overview[actor_rel].push(result);
                self.intermediate_chi_pon = Some(ChiPon {
                    consumed,
                    target_tile: pai,
                });
                self.pad_kawa_for_pon_or_daiminkan(actor, target);

                if actor_rel != 0 {
                    consumed.into_iter().for_each(|t| self.witness_tile(t));
                    result
                        .into_iter()
                        .for_each(|t| self.update_doras_owned(actor_rel, t));
                    self.can_w_riichi = false;
                    self.at_ippatsu = false;
                    return self.last_cans;
                }

                self.last_cans.can_discard = true;
                self.is_menzen = false;
                self.tehai_len_div3 -= 1;
                // Marked explicitly as `None` to let `Agent` impls set
                // `tsumogiri` to false in the Dahai after Pon
                self.last_self_tsumo = None;

                self.update_doras_owned(0, pai);
                consumed
                    .into_iter()
                    .for_each(|t| self.move_tile(t, MoveType::FuuroConsume));
                self.pons.push(pai.deaka().as_u8());

                if self.tehai[pai.deaka().as_usize()] > 0 {
                    self.forbidden_tiles[pai.deaka().as_usize()] = true;
                }

                // NOTES: this is 3n+2
                // The shanten can change after pon, for example 122334789 pon 2.
                self.update_shanten();
                self.update_shanten_discards();
            }

            Event::Daiminkan {
                actor,
                target,
                consumed,
                pai,
            } => {
                let actor_rel = self.rel(actor);
                let mut result = array_vec!();
                result.extend_from_slice(&consumed);
                result.push(pai);
                self.fuuro_overview[actor_rel].push(result);
                self.intermediate_kan.push(pai);
                self.pad_kawa_for_pon_or_daiminkan(actor, target);
                self.kans_on_board += 1;

                if actor_rel != 0 {
                    consumed.into_iter().for_each(|t| self.witness_tile(t));
                    result
                        .into_iter()
                        .for_each(|t| self.update_doras_owned(actor_rel, t));
                    self.can_w_riichi = false;
                    self.at_ippatsu = false;
                    return self.last_cans;
                }

                self.at_rinshan = true;
                self.is_menzen = false;
                self.tehai_len_div3 -= 1;

                self.update_doras_owned(0, pai);
                consumed
                    .into_iter()
                    .for_each(|t| self.move_tile(t, MoveType::FuuroConsume));
                self.minkans.push(pai.deaka().as_u8());

                // The shanten number and the shape of tenpai (if any) may be
                // changed after a daiminkan.
                //
                // For example: 12223m 456p 12378s + 2m
                self.update_shanten();
                self.update_waits_and_furiten();
            }

            Event::Kakan { actor, pai, .. } => {
                let actor_rel = self.rel(actor);
                for fuuro in &mut self.fuuro_overview[actor_rel] {
                    if fuuro[0].deaka() == pai.deaka() {
                        fuuro.push(pai);
                        break;
                    }
                }
                self.intermediate_kan.push(pai);
                self.kans_on_board += 1;

                if actor_rel != 0 {
                    self.witness_tile(pai);
                    self.update_doras_owned(actor_rel, pai);
                    self.last_kawa_tile = Some(pai); // for getting winning tile in self.agari

                    // 槍槓
                    if !self.at_furiten && self.waits[pai.deaka().as_usize()] {
                        self.last_cans.can_ron_agari = true;
                        self.to_mark_same_cycle_furiten = Some(());
                        self.chankan_chance = Some(());
                    } else {
                        self.at_ippatsu = false;
                    }

                    return self.last_cans;
                }

                self.at_rinshan = true;
                self.move_tile(pai, MoveType::FuuroConsume);
                self.pons.retain(|&t| t != pai.deaka().as_u8());
                self.minkans.push(pai.deaka().as_u8());

                // The shanten number and the shape of tenpai (if any) may
                // be changed after an kakan, because the kan'd tile may
                // come from the existing hand.
                if self.next_shanten_discards[pai.deaka().as_usize()] {
                    self.shanten -= 1;
                } else if !self.keep_shanten_discards[pai.deaka().as_usize()] {
                    self.update_shanten();
                }
                self.update_waits_and_furiten();
            }

            Event::Ankan { actor, consumed } => {
                let actor_rel = self.rel(actor);
                let tile = consumed[0].deaka();
                self.ankan_overview[actor_rel].push(tile);
                self.intermediate_kan.push(tile);
                self.kans_on_board += 1;

                self.can_w_riichi = false;
                self.at_ippatsu = false;

                if actor_rel != 0 {
                    for t in consumed {
                        self.witness_tile(t);
                        self.update_doras_owned(actor_rel, t);
                    }
                    return self.last_cans;
                }

                self.at_rinshan = true;
                self.tehai_len_div3 -= 1;
                consumed
                    .into_iter()
                    .for_each(|t| self.move_tile(t, MoveType::FuuroConsume));
                self.ankans.push(tile.as_u8());

                if !self.riichi_accepted[0] {
                    // The shanten number and the shape of tenpai (if any) may
                    // be changed after an ankan. See the example in daiminkan.
                    self.update_shanten();
                    self.update_waits_and_furiten();
                }
            }

            Event::Dora { dora_marker } => {
                self.add_dora_indicator(dora_marker);
            }

            Event::Reach { actor } => {
                let actor_rel = self.rel(actor);
                self.riichi_declared[actor_rel] = true;
                if actor_rel == 0 {
                    // `self.is_w_riichi` should not be set at ReachAccepted as
                    // `self.can_w_riichi` will be set to `false` right after
                    // the Dahai.
                    self.is_w_riichi = self.can_w_riichi;
                    self.last_cans.can_discard = true;
                }
            }

            Event::ReachAccepted { actor } => {
                let actor_rel = self.rel(actor);
                self.riichi_accepted[actor_rel] = true;
                self.scores[actor_rel] -= 1000;
                self.kyotaku += 1;
                self.update_rank();
                if actor_rel == 0 {
                    self.at_ippatsu = true;
                }
            }

            _ => (),
        };

        self.last_cans
    }

    pub(super) const fn rel(&self, actor: u8) -> usize {
        ((actor + 4 - self.player_id) % 4) as usize
    }

    /// Updates `tiles_seen` and `doras_seen`.
    pub(super) fn witness_tile(&mut self, tile: Tile) {
        let tile_id = tile.deaka().as_usize();
        self.tiles_seen[tile_id] += 1;
        self.doras_seen += self.dora_factor[tile_id];
        if tile.is_aka() {
            self.doras_seen += 1;
        }
    }

    /// Updates `akas_in_hand` and `doras_owned`, but does not update
    /// `tiles_seen` or `doras_seen`.
    pub(super) fn move_tile(&mut self, tile: Tile, move_type: MoveType) {
        if tile.is_aka() {
            let aka_id = tile.as_usize() - tuz!(5mr);
            match move_type {
                MoveType::Tsumo => {
                    self.akas_in_hand[aka_id] = true;
                    self.doras_owned[0] += 1;
                }
                MoveType::Discard => {
                    self.akas_in_hand[aka_id] = false;
                    self.doras_owned[0] -= 1;
                }
                MoveType::FuuroConsume => {
                    self.akas_in_hand[aka_id] = false;
                }
            }
        }

        let tile_id = tile.deaka().as_usize();
        match move_type {
            MoveType::Tsumo => {
                self.tehai[tile_id] += 1;
                self.doras_owned[0] += self.dora_factor[tile_id];
            }
            MoveType::Discard => {
                self.tehai[tile_id] -= 1;
                self.doras_owned[0] -= self.dora_factor[tile_id];
            }
            MoveType::FuuroConsume => {
                self.tehai[tile_id] -= 1;
            }
        }
    }

    /// Updates `dora_indicators`, witness the dora indicator itself and
    /// recounts doras (`doras_seen` and `doras_owned`) based on all the seen
    /// tiles.
    pub(super) fn add_dora_indicator(&mut self, tile: Tile) {
        self.dora_indicators.push(tile);

        // Witness the tile so it can be added to `tiles_seen`, possibly also to
        // `doras_seen`. This must be done before adding `dora_factor`.
        self.witness_tile(tile);

        let next = tile.next();
        self.dora_factor[next.as_usize()] += 1;

        // Count new dora in my tehai
        self.doras_owned[0] += self.tehai[next.as_usize()];

        // Count new dora in everyone's fuuro
        for i in 0..4 {
            let mut new_dora_count_in_fuuro = self.fuuro_overview[i]
                .iter()
                .flatten()
                .filter(|t| t.deaka() == next)
                .count() as u8;
            if self.ankan_overview[i].contains(&next) {
                new_dora_count_in_fuuro += 4;
            }
            self.doras_owned[i] += new_dora_count_in_fuuro;
        }

        // Add `doras_seen` based on `tiles_seen`
        self.doras_seen += self.tiles_seen[next.as_usize()];
    }

    pub(super) fn pad_kawa_for_pon_or_daiminkan(&mut self, abs_actor: u8, abs_target: u8) {
        let mut i = (abs_target + 1) % 4;
        while i != abs_actor {
            let rel = self.rel(i);
            self.kawa[rel].push(None);
            i = (i + 1) % 4;
        }
    }

    pub(super) fn pad_kawa_at_start(&mut self) {
        self.kawa
            .iter_mut()
            .take(self.oya as usize)
            .for_each(|kawa| kawa.push(None));
    }

    pub(super) fn set_can_chi_from_tile(&mut self, tile: Tile) {
        self.last_cans.can_chi_low = false;
        self.last_cans.can_chi_mid = false;
        self.last_cans.can_chi_high = false;

        let tile_id = tile.deaka().as_usize();
        let literal_num = tile_id % 9 + 1;

        // it considered case like 1111234 where you cannot chi 14
        if literal_num <= 7 && self.tehai[tile_id + 1] > 0 && self.tehai[tile_id + 2] > 0 {
            // TODO: check the conditions only when self.shanten == 0?
            let mut tehai_after = self.tehai;
            tehai_after[tile_id] = 0;
            tehai_after[tile_id + 1] -= 1;
            tehai_after[tile_id + 2] -= 1;
            if literal_num < 7 {
                tehai_after[tile_id + 3] = 0;
            }
            self.last_cans.can_chi_low = tehai_after.iter().any(|&t| t > 0);
        }

        if matches!(literal_num, 2..=8)
            && self.tehai[tile_id - 1] > 0
            && self.tehai[tile_id + 1] > 0
        {
            let mut tehai_after = self.tehai;
            tehai_after[tile_id] = 0;
            tehai_after[tile_id - 1] -= 1;
            tehai_after[tile_id + 1] -= 1;
            self.last_cans.can_chi_mid = tehai_after.iter().any(|&t| t > 0);
        }

        if literal_num >= 3 && self.tehai[tile_id - 2] > 0 && self.tehai[tile_id - 1] > 0 {
            let mut tehai_after = self.tehai;
            tehai_after[tile_id] = 0;
            tehai_after[tile_id - 2] -= 1;
            tehai_after[tile_id - 1] -= 1;
            if literal_num > 3 {
                tehai_after[tile_id - 3] = 0;
            }
            self.last_cans.can_chi_high = tehai_after.iter().any(|&t| t > 0);
        }
    }

    /// Can be called at either 3n+1 or 3n+2.
    ///
    /// For 3n+2, the return value of `shanten::calc_all` may be `-1`. We don't
    /// allow `-1` and it will be written as `0` in order for
    /// `_shanten_discards` to be calculated properly.
    pub(super) fn update_shanten(&mut self) {
        self.shanten = shanten::calc_all(&self.tehai, self.tehai_len_div3).max(0);
        debug_assert!(matches!(self.shanten, 0..=6));
    }

    /// Must be called at 3n+2.
    pub(super) fn update_shanten_discards(&mut self) {
        assert!(self.last_cans.can_discard, "tehai is not 3n+2");

        self.next_shanten_discards.fill(false);
        self.keep_shanten_discards.fill(false);
        self.has_next_shanten_discard = false;

        // benchmark result indicates it is too trivial to use rayon here.
        for tile_id in self
            .tehai
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(t, _)| t)
        {
            let mut tehai = self.tehai;
            tehai[tile_id] -= 1;
            let shanten_after = shanten::calc_all(&tehai, self.tehai_len_div3);
            match shanten_after.cmp(&self.shanten) {
                Ordering::Less => {
                    self.next_shanten_discards[tile_id] = true;
                    self.has_next_shanten_discard = true;
                }
                Ordering::Equal => {
                    self.keep_shanten_discards[tile_id] = true;
                }
                _ => (),
            };
        }
    }

    /// Caller must assure current tehai is 3n+1, and `self.shanten` must be up
    /// to date and correct.
    pub(super) fn update_waits_and_furiten(&mut self) {
        assert!(!self.last_cans.can_discard, "tehai is not 3n+1");

        // Reset the furiten flag here for:
        // 1. clearing same-cycle furiten
        // 2. the fact that furiten is nonsense if we are no longer tenpai
        self.at_furiten = false;
        self.waits.fill(false);

        if self.shanten > 0 {
            return;
        }

        for (t, v) in self.waits.iter_mut().enumerate() {
            if self.tehai[t] == 4 {
                // Cannot wait, not even furiten for the 5th tile.
                //
                // However waiting for the 5th tile with 4 of them lying in the
                // kawa or fuuro makes a valid furiten.
                //
                // Note that although [karaten] is not considered as a wait and
                // thus will not be written to the `waits` in this impl anyways,
                // it is still a valid ryukyoku tenpai in our rule spec.
                continue;
            }
            let mut tehai_after = self.tehai;
            tehai_after[t] += 1;

            if shanten::calc_all(&tehai_after, self.tehai_len_div3) == -1 {
                // furiten is not affected by `tiles_seen`
                self.at_furiten |= self.discarded_tiles[t];
                *v = self.tiles_seen[t] < 4;
            }
        }
    }

    pub(super) fn update_doras_owned(&mut self, actor_rel: usize, tile: Tile) {
        self.doras_owned[actor_rel] += self.dora_factor[tile.deaka().as_usize()];
        if tile.is_aka() {
            self.doras_owned[actor_rel] += 1;
        }
    }

    pub(super) fn update_rank(&mut self) {
        self.rank = self.get_rank(self.scores);
    }

    pub(super) fn get_rank(&self, mut scores_rel: [i32; 4]) -> u8 {
        let scores_abs = {
            scores_rel.rotate_right(self.player_id as usize);
            scores_rel
        };
        Rankings::new(scores_abs).rank_by_player[self.player_id as usize]
    }
}
