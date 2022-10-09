use super::PlayerState;
use crate::algo::agari::AgariCalculator;
use crate::algo::point::Point;
use crate::algo::shanten;
use crate::tile::Tile;
use crate::vec_ops::vec_add_assign;
use crate::{must_tile, t, tuz};

use anyhow::{ensure, Context, Result};
use tinyvec::array_vec;

impl PlayerState {
    /// Used by `BoardState` to check if a player is making 4 kans on his own.
    #[must_use]
    pub fn kans_count(&self) -> usize {
        self.minkans.len() + self.ankans.len()
    }

    /// Used by `Agent` impls, must be called at 3n+2.
    #[must_use]
    pub fn discard_candidates(&self) -> [bool; 34] {
        let full = self.discard_candidates_aka();
        let mut ret = [false; 34];
        ret.copy_from_slice(&full[..34]);
        ret[tuz!(5m)] |= full[tuz!(5mr)];
        ret[tuz!(5s)] |= full[tuz!(5sr)];
        ret[tuz!(5p)] |= full[tuz!(5pr)];
        ret
    }

    /// Aka dora covered version of `discard_candidates`.
    #[must_use]
    pub fn discard_candidates_aka(&self) -> [bool; 37] {
        assert!(self.last_cans.can_discard, "tehai is not 3n+2");

        let mut ret = [false; 37];

        if self.riichi_accepted[0] {
            let last_self_tsumo = self
                .last_self_tsumo
                .expect("riichi accepted without last self tsumo");
            ret[last_self_tsumo.as_usize()] = true;
            return ret;
        }

        for (i, count) in self.tehai.iter().copied().enumerate() {
            if count == 0 {
                continue;
            }

            ret[i] = if self.riichi_declared[0] {
                if self.shanten == 1 {
                    self.next_shanten_discards[i]
                } else {
                    // shanten must be 0 here according to the rule
                    self.keep_shanten_discards[i]
                }
            } else {
                !self.forbidden_tiles[i]
            };
        }

        if ret[tuz!(5m)] && self.akas_in_hand[0] {
            ret[tuz!(5mr)] = true;
            ret[tuz!(5m)] = self.tehai[tuz!(5m)] > 1;
        }
        if ret[tuz!(5p)] && self.akas_in_hand[1] {
            ret[tuz!(5pr)] = true;
            ret[tuz!(5p)] = self.tehai[tuz!(5p)] > 1;
        }
        if ret[tuz!(5s)] && self.akas_in_hand[2] {
            ret[tuz!(5sr)] = true;
            ret[tuz!(5s)] = self.tehai[tuz!(5s)] > 1;
        }

        ret
    }

    /// Must be called at 3n+2.
    ///
    /// The return value indicates the tiles which can make the hand tenpai for
    /// real after being discarded, with the number of future tenpai tiles left
    /// and furiten considered, without depending on any incidental yaku, and is
    /// not affected by the riichi status of the player.
    #[must_use]
    pub fn discard_candidates_with_unconditional_tenpai(&self) -> [bool; 34] {
        let full = self.discard_candidates_with_unconditional_tenpai_aka();
        let mut ret = [false; 34];
        ret.copy_from_slice(&full[..34]);
        ret[tuz!(5m)] |= full[tuz!(5mr)];
        ret[tuz!(5s)] |= full[tuz!(5sr)];
        ret[tuz!(5p)] |= full[tuz!(5pr)];
        ret
    }

    /// Aka dora covered version of `discard_candidates_with_unconditional_tenpai`.
    #[must_use]
    pub fn discard_candidates_with_unconditional_tenpai_aka(&self) -> [bool; 37] {
        assert!(self.last_cans.can_discard, "tehai is not 3n+2");

        let mut ret = [false; 37];

        if self.tiles_left == 0 // haitei
            || self.shanten > 1 // impossible to discard-to-tenpai
            || self.shanten == 1 && !self.has_next_shanten_discard
        {
            return ret;
        }

        if let Some(last_self_tsumo) = self.last_self_tsumo {
            if self.waits[last_self_tsumo.deaka().as_usize()] {
                // already agari and any discard will result in furiten
                return ret;
            }
            if self.riichi_accepted[0] {
                if !self.at_furiten {
                    // already riichi and is not furiten (which is forever)
                    ret[last_self_tsumo.as_usize()] = true;
                }
                return ret;
            }
        } else if shanten::calc_all(&self.tehai, self.tehai_len_div3) == -1 {
            // Ditto but for discard after chi/pon
            return ret;
        }

        let tenpai_discards = if self.shanten == 1 {
            self.next_shanten_discards
        } else {
            self.keep_shanten_discards
        };

        // Replace and test
        tenpai_discards
            .iter()
            .copied()
            .enumerate()
            .filter(|&(tid, b)| b && !self.forbidden_tiles[tid])
            .for_each(|(discard, _)| {
                let mut tehai_3n1 = self.tehai;
                tehai_3n1[discard] -= 1;

                for (tsumo, seen) in self.tiles_seen.iter().copied().enumerate() {
                    if tsumo == discard || tehai_3n1[tsumo] == 4 {
                        continue;
                    }

                    let mut tehai_3n2 = tehai_3n1;
                    tehai_3n2[tsumo] += 1;
                    if shanten::calc_all(&tehai_3n2, self.tehai_len_div3) > -1 {
                        continue;
                    }

                    // Furiten
                    if self.discarded_tiles[tsumo] {
                        ret[discard] = false;
                        break;
                    }

                    // Must be placed after the furiten check above
                    if seen == 4 || ret[discard] {
                        continue;
                    }

                    let agari_calc = AgariCalculator {
                        tehai: &tehai_3n2,
                        is_menzen: self.is_menzen,
                        chis: &self.chis,
                        pons: &self.pons,
                        minkans: &self.minkans,
                        ankans: &self.ankans,
                        bakaze: self.bakaze.as_u8(),
                        jikaze: self.jikaze.as_u8(),
                        winning_tile: tsumo as u8,
                        is_ron: true,
                    };
                    ret[discard] = agari_calc.has_yaku();
                }
            });

        if ret[tuz!(5m)] && self.akas_in_hand[0] {
            ret[tuz!(5mr)] = true;
            ret[tuz!(5m)] = self.tehai[tuz!(5m)] > 1;
        }
        if ret[tuz!(5p)] && self.akas_in_hand[1] {
            ret[tuz!(5pr)] = true;
            ret[tuz!(5p)] = self.tehai[tuz!(5p)] > 1;
        }
        if ret[tuz!(5s)] && self.akas_in_hand[2] {
            ret[tuz!(5sr)] = true;
            ret[tuz!(5s)] = self.tehai[tuz!(5s)] > 1;
        }

        ret
    }

    #[inline]
    #[must_use]
    pub fn yaokyuu_kind_count(&self) -> u8 {
        tuz![1m, 9m, 1p, 9p, 1s, 9s, E, S, W, N, P, F, C]
            .iter()
            .map(|&i| self.tehai[i].min(1))
            .sum()
    }

    #[inline]
    #[must_use]
    pub fn rule_based_ryukyoku(&self) -> bool {
        if !self.last_cans.can_ryukyoku {
            return false;
        }
        self.rule_based_ryukyoku_slow()
    }

    #[inline(never)]
    fn rule_based_ryukyoku_slow(&self) -> bool {
        // Do not ryukyoku if the hand is already <= 2 shanten.
        if shanten::calc_all(&self.tehai, self.tehai_len_div3) <= 2 {
            return false;
        }

        // Ryukyoku if we are in the west round, because we usually don't need a
        // big hand to win.
        if self.bakaze == t!(W) {
            return true;
        }

        if self.is_all_last {
            // Ryukyoku if it is all-last and we are oya or we are not the last,
            // because it is hard to decide whether it is appropriate to not
            // ryukyoku.
            if self.oya == 0 || self.rank < 3 {
                return true;
            }

            // At all-last, we are the last and we are not oya. If even a
            // haneman tsumo cannot let us avoid the last, then do not ryukyoku.
            let mut scores = [-3000 - self.honba as i32 * 300; 4];
            scores[0] = 12000 + self.kyotaku as i32 * 1000 + self.honba as i32 * 300;
            scores[self.oya as usize] = -6000 - self.honba as i32 * 300;
            vec_add_assign(&mut scores, &self.scores);
            return self.get_rank(scores) < 3;
        }

        // Do not ryukyoku if we have >= 10 yaokyuu tiles.
        if self.yaokyuu_kind_count() >= 10 {
            return false;
        }

        // Do not ryukyoku if we have all the jihai kinds.
        if self.tehai[3 * 9..].iter().all(|&c| c > 0) {
            return false;
        }

        // Ryukyoku otherwise.
        true
    }

    #[inline]
    #[must_use]
    pub fn rule_based_agari(&self) -> bool {
        if !self.last_cans.can_agari() {
            return false;
        }
        self.rule_based_agari_slow(
            self.last_cans.can_ron_agari,
            self.rel(self.last_cans.target_actor),
        )
    }

    #[inline(never)]
    fn rule_based_agari_slow(&self, is_ron: bool, target_rel: usize) -> bool {
        // Agari if it is not yet all-last, or we are oya ourselves, or we are
        // not the last place at all.
        if !self.is_all_last || self.oya == 0 || self.rank < 3 {
            return true;
        }

        if self.bakaze == t!(W) {
            // Agari if we are in the west round but it is not yet the real
            // all-last (W4).
            if self.kyoku < 3 {
                return true;
            }
        } else if self.scores.iter().all(|&s| s < 30000) {
            // Agari if 西入 is possible. Note that this condition is sound but
            // not complete.
            return true;
        }

        // Calculate the max theoretical score we can achieve through this agari.
        let max_win_point = if self.riichi_accepted[0] {
            let mut tehai_full = self.tehai;
            for t in &self.ankan_overview[0] {
                tehai_full[t.as_usize()] += 4;
            }

            let mut tehai_ordered_by_count: Vec<_> = tehai_full
                .iter()
                .enumerate()
                .filter(|&(_, &c)| c > 0)
                .collect();
            tehai_ordered_by_count.sort_unstable_by(|(_, l), (_, r)| r.cmp(l));

            // Try possible uradoras one by one, starting from the most valuable one
            let mut tiles_seen = self.tiles_seen;
            let mut ura_indicators = array_vec!([_; 5]);
            tehai_ordered_by_count
                .into_iter()
                .map(|(t, _)| must_tile!(t).prev())
                .take_while(|&ura| loop {
                    if ura_indicators.len() >= self.dora_indicators.len() {
                        // Break out of all loops.
                        return false;
                    }
                    if tiles_seen[ura.as_usize()] >= 4 {
                        // Try the next most-valuable possible uradora.
                        return true;
                    }
                    ura_indicators.push(ura);
                    tiles_seen[ura.as_usize()] += 1;
                })
                .for_each(drop);

            // `unwrap` is safe because there is a condition guard in
            // `rule_based_agari`.
            self.agari_points(is_ron, &ura_indicators).unwrap()
        } else {
            // ditto
            self.agari_points(is_ron, &[]).unwrap()
        };

        // Calculate the best post-hora situation for us.
        let mut exp_scores = self.scores;
        if is_ron {
            exp_scores[0] +=
                max_win_point.ron + self.kyotaku as i32 * 1000 + self.honba as i32 * 300;
            exp_scores[target_rel] -= max_win_point.ron + self.honba as i32 * 300;
        } else {
            // The player must be ko here.
            exp_scores[0] += max_win_point.tsumo_ko * 2
                + max_win_point.tsumo_oya
                + self.kyotaku as i32 * 1000
                + self.honba as i32 * 300;
            exp_scores
                .iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(idx, s)| {
                    if idx as u8 == self.oya {
                        *s -= max_win_point.tsumo_oya + self.honba as i32 * 100;
                    } else {
                        *s -= max_win_point.tsumo_ko + self.honba as i32 * 100;
                    }
                });
        }

        // The prerequisite `!(self.bakaze == t!(W) && self.kyoku == 3)` has
        // already been checked at the beginning.
        //
        // Agari if 西入 or keeping 西入 is possible. This condition is sound
        // and complete.
        if exp_scores.iter().all(|&s| s < 30000) {
            return true;
        }

        // Agari if the best post-hora situation in theory will make us avoid
        // taking the last place.
        self.get_rank(exp_scores) < 3
    }

    /// Err is returned if the hand cannot agari, or cannot retrieve the winning
    /// tile.
    ///
    /// This function should be called immediately, otherwise the state may
    /// change.
    ///
    /// `ura_indicators` is only used when the actor has an accepted riichi.
    pub fn agari_points(&self, is_ron: bool, ura_indicators: &[Tile]) -> Result<Point> {
        ensure!(
            is_ron && self.last_cans.can_ron_agari || self.last_cans.can_tsumo_agari,
            "cannot agari"
        );

        // 天和, 地和 are special cases that are handled individually, and there
        // is no multi yakuman for these two.
        if !is_ron && self.can_w_riichi {
            return Ok(Point::yakuman(self.oya == 0, 1));
        }

        let winning_tile = if is_ron {
            self.last_kawa_tile
        } else {
            self.last_self_tsumo
        }
        .context("cannot find the winning tile")?;

        let additional_hans = if is_ron {
            [
                self.riichi_accepted[0],       // 立直
                self.is_w_riichi,              // 両立直
                self.at_ippatsu,               // 一发
                self.tiles_left == 0,          // 河底撈魚
                self.chankan_chance.is_some(), // 槍槓
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u8
        } else {
            [
                self.riichi_accepted[0],                  // 立直
                self.is_w_riichi,                         // 両立直
                self.at_ippatsu,                          // 一发
                self.is_menzen,                           // 門前清自摸和
                self.tiles_left == 0 && !self.at_rinshan, // 海底摸月
                self.at_rinshan,                          // 嶺上開花
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u8
        };

        let mut tehai = self.tehai;
        let mut final_doras_owned = self.doras_owned[0];
        if is_ron {
            let tid = winning_tile.deaka().as_usize();
            tehai[tid] += 1;
            final_doras_owned += self.dora_factor[tid];
            if winning_tile.is_aka() {
                final_doras_owned += 1;
            };
        }
        if self.riichi_accepted[0] {
            final_doras_owned += ura_indicators
                .iter()
                .map(|&ura| {
                    let next = ura.next();
                    let mut count = tehai[next.as_usize()];
                    if self.ankan_overview[0].contains(&next) {
                        count += 4;
                    }
                    count
                })
                .sum::<u8>();
        }

        let agari_calc = AgariCalculator {
            tehai: &tehai,
            is_menzen: self.is_menzen,
            chis: &self.chis,
            pons: &self.pons,
            minkans: &self.minkans,
            ankans: &self.ankans,
            bakaze: self.bakaze.as_u8(),
            jikaze: self.jikaze.as_u8(),
            winning_tile: winning_tile.deaka().as_u8(),
            is_ron,
        };
        let agari = agari_calc
            .agari(additional_hans, final_doras_owned)
            .context("not a hora hand")?;

        Ok(agari.into_point(self.oya == 0))
    }
}
