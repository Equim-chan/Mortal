const OYA_RON_POINTS: &[&[i32]] = &[
    &[],
    &[],
    &[],
    &[15, 29, 58, 116], // 30
    &[20, 39, 77],      // 40
    &[24, 48, 96],      // 50
    &[],
    &[34, 68], // 70
    &[],
    &[44, 87], // 90
    &[],
    &[53, 106], // 110
];

const OYA_TSUMO_POINTS: &[&[i32]] = &[
    &[],
    &[],
    &[],
    &[5, 10, 20, 39], // 30
    &[7, 13, 26],     // 40
    &[8, 16, 32],     // 50
    &[],
    &[12, 23], // 70
    &[],
    &[15, 29], // 90
    &[],
    &[i32::MIN, 36], // 110
];

const KODOMO_RON_POINTS: &[&[i32]] = &[
    &[],
    &[],
    &[],
    &[10, 20, 39, 77], // 30
    &[13, 26, 52],     // 40
    &[16, 32, 64],     // 50
    &[],
    &[23, 45], // 70
    &[],
    &[29, 58], // 90
    &[],
    &[36, 71], // 110
];

const KODOMO_TSUMO_POINTS: &[&[(i32, i32)]] = &[
    &[],
    &[],
    &[],
    &[(3, 5), (5, 10), (10, 20), (20, 39)], // 30
    &[(4, 7), (7, 13), (13, 26)],           // 40
    &[(4, 8), (8, 16), (16, 32)],           // 50
    &[],
    &[(6, 12), (12, 13)], // 70
    &[],
    &[(8, 15), (15, 29)], // 90
    &[],
    &[(i32::MIN, i32::MIN), (18, 36)], // 110
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub ron: i32,
    pub tsumo_oya: i32,
    pub tsumo_ko: i32,
}

impl Point {
    #[inline]
    #[must_use]
    pub const fn tsumo_total(self, is_oya: bool) -> i32 {
        if is_oya {
            self.tsumo_ko * 3
        } else {
            self.tsumo_ko * 2 + self.tsumo_oya
        }
    }

    #[inline]
    #[must_use]
    pub const fn mangan(is_oya: bool) -> Self {
        if is_oya {
            Self {
                ron: 12000,
                tsumo_oya: 0,
                tsumo_ko: 4000,
            }
        } else {
            Self {
                ron: 8000,
                tsumo_oya: 4000,
                tsumo_ko: 2000,
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn haneman(is_oya: bool) -> Self {
        if is_oya {
            Self {
                ron: 18000,
                tsumo_oya: 0,
                tsumo_ko: 6000,
            }
        } else {
            Self {
                ron: 12000,
                tsumo_oya: 6000,
                tsumo_ko: 3000,
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn baiman(is_oya: bool) -> Self {
        if is_oya {
            Self {
                ron: 24000,
                tsumo_oya: 0,
                tsumo_ko: 8000,
            }
        } else {
            Self {
                ron: 16000,
                tsumo_oya: 8000,
                tsumo_ko: 4000,
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn sanbaiman(is_oya: bool) -> Self {
        if is_oya {
            Self {
                ron: 36000,
                tsumo_oya: 0,
                tsumo_ko: 12000,
            }
        } else {
            Self {
                ron: 24000,
                tsumo_oya: 12000,
                tsumo_ko: 6000,
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn yakuman(is_oya: bool, count: i32) -> Self {
        if is_oya {
            Self {
                ron: 48000 * count,
                tsumo_oya: 0,
                tsumo_ko: 16000 * count,
            }
        } else {
            Self {
                ron: 32000 * count,
                tsumo_oya: 16000 * count,
                tsumo_ko: 8000 * count,
            }
        }
    }

    const fn mangan_up(han: u8, is_oya: bool) -> Self {
        match han {
            3..=5 => Self::mangan(is_oya),
            6..=7 => Self::haneman(is_oya),
            8..=10 => Self::baiman(is_oya),
            11..=12 => Self::sanbaiman(is_oya),
            _ => Self::yakuman(is_oya, 1),
        }
    }

    /// If `is_oya` holds, the `tsumo_oya` of the return value will always be `0`.
    #[must_use]
    pub fn calc(fu: u8, han: u8, is_oya: bool) -> Self {
        if han >= 5 || fu >= 40 && han >= 4 {
            return Self::mangan_up(han, is_oya);
        }

        let (key, idx) = match fu {
            20 | 25 => (fu as usize / 5, han as usize - 2),
            60 | 80 | 100 => (fu as usize / 20, han as usize),
            _ => (fu as usize / 10, han as usize - 1),
        };

        let ron_table = if is_oya {
            OYA_RON_POINTS
        } else {
            KODOMO_RON_POINTS
        };

        if let Some(ron) = ron_table[key].get(idx).copied() {
            if is_oya {
                let tsumo = OYA_TSUMO_POINTS[key][idx];
                Self {
                    ron: ron * 100,
                    tsumo_oya: 0,
                    tsumo_ko: tsumo * 100,
                }
            } else {
                let (tsumo_ko, tsumo_oya) = KODOMO_TSUMO_POINTS[key][idx];
                Self {
                    ron: ron * 100,
                    tsumo_oya: tsumo_oya * 100,
                    tsumo_ko: tsumo_ko * 100,
                }
            }
        } else {
            Self::mangan_up(han, is_oya)
        }
    }
}
