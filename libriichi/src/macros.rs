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
        0u8
    };
    (2m) => {
        1u8
    };
    (3m) => {
        2u8
    };
    (4m) => {
        3u8
    };
    (5m) => {
        4u8
    };
    (6m) => {
        5u8
    };
    (7m) => {
        6u8
    };
    (8m) => {
        7u8
    };
    (9m) => {
        8u8
    };

    (1p) => {
        9u8
    };
    (2p) => {
        10u8
    };
    (3p) => {
        11u8
    };
    (4p) => {
        12u8
    };
    (5p) => {
        13u8
    };
    (6p) => {
        14u8
    };
    (7p) => {
        15u8
    };
    (8p) => {
        16u8
    };
    (9p) => {
        17u8
    };

    (1s) => {
        18u8
    };
    (2s) => {
        19u8
    };
    (3s) => {
        20u8
    };
    (4s) => {
        21u8
    };
    (5s) => {
        22u8
    };
    (6s) => {
        23u8
    };
    (7s) => {
        24u8
    };
    (8s) => {
        25u8
    };
    (9s) => {
        26u8
    };

    (E) => {
        27u8
    };
    (S) => {
        28u8
    };
    (W) => {
        29u8
    };
    (N) => {
        30u8
    };
    (P) => {
        31u8
    };
    (F) => {
        32u8
    };
    (C) => {
        33u8
    };

    (5mr) => {
        34u8
    };
    (5pr) => {
        35u8
    };
    (5sr) => {
        36u8
    };

    (?) => {
        37u8
    };

    [$($s:tt),* $(,)?] => {
        [$($crate::tu8!($s)),*]
    };
}

/// Used for making const tile IDs in usize.
#[macro_export]
macro_rules! tuz {
    ($s:tt) => {
        $crate::tu8!($s) as usize
    };
    [$($s:tt),* $(,)?] => {
        [$($crate::tuz!($s)),*]
    };
}

/// Used for making const tiles.
#[macro_export]
macro_rules! t {
    ($s:tt) => {
        unsafe { $crate::tile::Tile::new_unchecked($crate::tu8!($s)) }
    };
    [$($s:tt),* $(,)?] => {
        [$($crate::t!($s)),*]
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
    ($($id:tt)*) => {
        $crate::tile::Tile::try_from($($id)*).unwrap()
    };
}

#[cfg(test)]
mod test {
    /// ```rust,compile_fail
    /// use riichi::tu8;
    ///
    /// let t = tu8!(0m);
    /// ```
    struct _CompileFail;

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
