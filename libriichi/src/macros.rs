/// Used for making const tile IDs in u8.
///
/// ```
/// use riichi::tu8;
///
/// assert_eq!(tu8!(E), 27u8);
/// ```
#[macro_export]
macro_rules! tu8 {
    (1m) => {
        0_u8
    };
    (2m) => {
        1_u8
    };
    (3m) => {
        2_u8
    };
    (4m) => {
        3_u8
    };
    (5m) => {
        4_u8
    };
    (6m) => {
        5_u8
    };
    (7m) => {
        6_u8
    };
    (8m) => {
        7_u8
    };
    (9m) => {
        8_u8
    };

    (1p) => {
        9_u8
    };
    (2p) => {
        10_u8
    };
    (3p) => {
        11_u8
    };
    (4p) => {
        12_u8
    };
    (5p) => {
        13_u8
    };
    (6p) => {
        14_u8
    };
    (7p) => {
        15_u8
    };
    (8p) => {
        16_u8
    };
    (9p) => {
        17_u8
    };

    (1s) => {
        18_u8
    };
    (2s) => {
        19_u8
    };
    (3s) => {
        20_u8
    };
    (4s) => {
        21_u8
    };
    (5s) => {
        22_u8
    };
    (6s) => {
        23_u8
    };
    (7s) => {
        24_u8
    };
    (8s) => {
        25_u8
    };
    (9s) => {
        26_u8
    };

    (E) => {
        27_u8
    };
    (S) => {
        28_u8
    };
    (W) => {
        29_u8
    };
    (N) => {
        30_u8
    };
    (P) => {
        31_u8
    };
    (F) => {
        32_u8
    };
    (C) => {
        33_u8
    };

    (5mr) => {
        34_u8
    };
    (5pr) => {
        35_u8
    };
    (5sr) => {
        36_u8
    };

    (?) => {
        37_u8
    };

    ($first:tt, $($left:tt),*) => {
        [$crate::tu8!($first), $($crate::tu8!($left)),*]
    };

    ($($_:tt)*) => {
        ::std::compile_error!("invalid tile pattern");
    }
}

/// Used for making const tile IDs in usize.
#[macro_export]
macro_rules! tuz {
    ($s:tt) => {
        $crate::tu8!($s) as usize
    };
    ($first:tt, $($left:tt),*) => {
        [$crate::tuz!($first), $($crate::tuz!($left)),*]
    };
}

/// Used for making const tiles.
#[macro_export]
macro_rules! t {
    ($s:tt) => {
        // All possible values of `tu8!` are valid for `Tile`.
        $crate::tile::Tile::new_unchecked($crate::tu8!($s))
    };
    ($first:tt, $($left:tt),*) => {
        [$crate::t!($first), $($crate::t!($left)),*]
    };
}

/// A handy macro for matching a `u8` against const tile IDs.
#[macro_export]
macro_rules! matches_tu8 {
    ($o:expr, $($s:tt)|* $(|)?) => {
        matches!($o, $($crate::tu8!($s))|*)
    };
}

/// Used for making non-const tiles.
///
/// # Panics
/// Panics if the input is not a valid tile.
///
/// ```rust,should_panic
/// use riichi::{must_tile, tu8};
///
/// let t = must_tile!(tu8!(?) + 1);
/// ```
#[macro_export]
macro_rules! must_tile {
    ($($id:tt)*) => {{
        #[cfg(debug_assertions)]
        { $crate::tile::Tile::try_from($($id)*).unwrap() }
        #[cfg(not(debug_assertions))]
        { $crate::tile::Tile::new_unchecked(($($id)*) as u8) }
    }};
}

#[cfg(doctest)]
/// ```rust,compile_fail
/// use riichi::tu8;
///
/// let t = tu8!(0m);
/// ```
struct _CompileFail;

#[cfg(test)]
mod test {
    #[test]
    fn syntax() {
        assert_eq!(t!(3s).as_usize(), tuz!(3s));
        assert_eq!(t!(5sr).as_u8(), tu8!(5sr));
        assert_eq!(t!(5m).akaize().as_u8(), tu8!(5mr));

        assert_eq!(tu8![8m,], [tu8!(8m)]);
        assert_eq!(tuz![P,], [tuz!(P)]);
        assert_eq!(t![N,], [t!(N)]);

        assert_eq!(tu8![2p, 5pr, S], [tu8!(2p), tu8!(5pr), tu8!(S)]);
        assert_eq!(tuz![E, 6m, ?], [tuz!(E), tuz!(6m), tuz!(?)]);
        assert_eq!(t![1m, 2p, 9s], [t!(1m), t!(2p), t!(9s)]);

        assert!(matches_tu8!(t!(E).as_u8(), 1m | E | ? | 5mr));
        assert!(!matches_tu8!(t!(3m).as_u8(), 1s | 7p | P));
    }

    #[test]
    fn completeness() {
        assert_eq!(t!(1m).to_string(), "1m");
        assert_eq!(t!(2m).to_string(), "2m");
        assert_eq!(t!(3m).to_string(), "3m");
        assert_eq!(t!(4m).to_string(), "4m");
        assert_eq!(t!(5m).to_string(), "5m");
        assert_eq!(t!(6m).to_string(), "6m");
        assert_eq!(t!(7m).to_string(), "7m");
        assert_eq!(t!(8m).to_string(), "8m");
        assert_eq!(t!(9m).to_string(), "9m");

        assert_eq!(t!(1p).to_string(), "1p");
        assert_eq!(t!(2p).to_string(), "2p");
        assert_eq!(t!(3p).to_string(), "3p");
        assert_eq!(t!(4p).to_string(), "4p");
        assert_eq!(t!(5p).to_string(), "5p");
        assert_eq!(t!(6p).to_string(), "6p");
        assert_eq!(t!(7p).to_string(), "7p");
        assert_eq!(t!(8p).to_string(), "8p");
        assert_eq!(t!(9p).to_string(), "9p");

        assert_eq!(t!(1s).to_string(), "1s");
        assert_eq!(t!(2s).to_string(), "2s");
        assert_eq!(t!(3s).to_string(), "3s");
        assert_eq!(t!(4s).to_string(), "4s");
        assert_eq!(t!(5s).to_string(), "5s");
        assert_eq!(t!(6s).to_string(), "6s");
        assert_eq!(t!(7s).to_string(), "7s");
        assert_eq!(t!(8s).to_string(), "8s");
        assert_eq!(t!(9s).to_string(), "9s");

        assert_eq!(t!(E).to_string(), "E");
        assert_eq!(t!(S).to_string(), "S");
        assert_eq!(t!(W).to_string(), "W");
        assert_eq!(t!(N).to_string(), "N");
        assert_eq!(t!(P).to_string(), "P");
        assert_eq!(t!(F).to_string(), "F");
        assert_eq!(t!(C).to_string(), "C");

        assert_eq!(t!(5mr).to_string(), "5mr");
        assert_eq!(t!(5pr).to_string(), "5pr");
        assert_eq!(t!(5sr).to_string(), "5sr");

        assert_eq!(t!(?).to_string(), "?");
    }
}
