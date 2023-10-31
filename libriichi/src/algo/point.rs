#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub ron: i32,
    pub tsumo_ko: i32,
    pub tsumo_oya: i32,
}

impl Point {
    /// Panics if the combinition is not possible.
    ///
    /// If `is_oya` holds, the `tsumo_oya` of the return value will always be `0`.
    #[must_use]
    pub fn calc(is_oya: bool, fu: u8, han: u8) -> Self {
        let (ron, tsumo_ko, tsumo_oya) = if is_oya {
            match (fu, han) {
                (20, 2) | (40, 1) => (2000, 700, 0),
                (20, 3) | (40, 2) | (80, 1) => (3900, 1300, 0),
                (20, 4) | (40, 3) | (80, 2) => (7700, 2600, 0),
                // ---
                (25, 2) | (50, 1) => (2400, 800, 0),
                (25, 3) | (50, 2) | (100, 1) => (4800, 1600, 0),
                (25, 4) | (50, 3) | (100, 2) => (9600, 3200, 0),
                // ---
                (30, 1) => (1500, 500, 0),
                (30, 2) | (60, 1) => (2900, 1000, 0),
                (30, 3) | (60, 2) => (5800, 2000, 0),
                (30, 4) | (60, 3) => (11600, 3900, 0),
                // ---
                (70, 1) => (3400, 1200, 0),
                (70, 2) => (6800, 2300, 0),
                // ---
                (90, 1) => (4400, 1500, 0),
                (90, 2) => (8700, 2900, 0),
                // ---
                // theoretical value, since 110/1 tsumo is not possible
                (110, 1) => (5300, 1800, 0),
                (110, 2) => (10600, 3600, 0),
                // ---
                (_, 5) | (40.., 4) | (70.., 3) => (12000, 4000, 0),
                (_, 6..=7) => (18000, 6000, 0),
                (_, 8..=10) => (24000, 8000, 0),
                (_, 11..=12) => (36000, 12000, 0),
                (_, 13..) => (48000, 16000, 0),
                _ => panic!("impossible combinition of {fu} fu and {han} han"),
            }
        } else {
            match (fu, han) {
                (20, 2) | (40, 1) => (1300, 400, 700),
                (20, 3) | (40, 2) | (80, 1) => (2600, 700, 1300),
                (20, 4) | (40, 3) | (80, 2) => (5200, 1300, 2600),
                // ---
                (25, 2) | (50, 1) => (1600, 400, 800),
                (25, 3) | (50, 2) | (100, 1) => (3200, 800, 1600),
                (25, 4) | (50, 3) | (100, 2) => (6400, 1600, 3200),
                // ---
                (30, 1) => (1000, 300, 500),
                (30, 2) | (60, 1) => (2000, 500, 1000),
                (30, 3) | (60, 2) => (3900, 1000, 2000),
                (30, 4) | (60, 3) => (7700, 2000, 3900),
                // ---
                (70, 1) => (2300, 600, 1200),
                (70, 2) => (4500, 1200, 2300),
                // ---
                (90, 1) => (2900, 800, 1500),
                (90, 2) => (5800, 1500, 2900),
                // ---
                // theoretical value, since 110/1 tsumo is not possible
                (110, 1) => (3600, 900, 1800),
                (110, 2) => (7100, 1800, 3600),
                // ---
                (_, 5) | (40.., 4) | (70.., 3) => (8000, 2000, 4000),
                (_, 6..=7) => (12000, 3000, 6000),
                (_, 8..=10) => (16000, 4000, 8000),
                (_, 11..=12) => (24000, 6000, 12000),
                (_, 13..) => (32000, 8000, 16000),
                _ => panic!("impossible combinition of {fu} fu and {han} han"),
            }
        };
        Self {
            ron,
            tsumo_ko,
            tsumo_oya,
        }
    }

    #[inline]
    #[must_use]
    pub const fn yakuman(is_oya: bool, count: i32) -> Self {
        if is_oya {
            Self {
                ron: 48000 * count,
                tsumo_ko: 16000 * count,
                tsumo_oya: 0,
            }
        } else {
            Self {
                ron: 32000 * count,
                tsumo_ko: 8000 * count,
                tsumo_oya: 16000 * count,
            }
        }
    }

    #[inline]
    #[must_use]
    pub const fn tsumo_total(self, is_oya: bool) -> i32 {
        if is_oya {
            self.tsumo_ko * 3
        } else {
            self.tsumo_ko * 2 + self.tsumo_oya
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::once;

    #[test]
    fn table() {
        for fu in (20..=110).step_by(10).chain(once(25)) {
            for han in 1..=14 {
                if han == 1 && fu < 30 {
                    continue;
                }

                let base_points = if han >= 13 {
                    8000
                } else if han >= 11 {
                    6000
                } else if han >= 8 {
                    4000
                } else if han >= 6 {
                    3000
                } else if han >= 5 {
                    2000
                } else {
                    (fu * 2_i32.pow(2 + han)).min(2000)
                };
                let get_points = |mult| (base_points * mult + 99) / 100 * 100;

                let points_ko = Point::calc(false, fu as u8, han as u8);
                assert_eq!(points_ko.tsumo_ko, get_points(1), "{fu}/{han}");
                assert_eq!(points_ko.tsumo_oya, get_points(2), "{fu}/{han}");
                assert_eq!(points_ko.ron, get_points(4), "{fu}/{han}");

                let points_oya = Point::calc(true, fu as u8, han as u8);
                assert_eq!(points_oya.tsumo_ko, get_points(2), "{fu}/{han}");
                assert_eq!(points_oya.ron, get_points(6), "{fu}/{han}");
            }
        }
    }
}
