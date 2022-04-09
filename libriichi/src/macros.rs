#[macro_export]
macro_rules! tu8 {
    (1m) => {
        0
    };
    (2m) => {
        1
    };
    (3m) => {
        2
    };
    (4m) => {
        3
    };
    (5m) => {
        4
    };
    (6m) => {
        5
    };
    (7m) => {
        6
    };
    (8m) => {
        7
    };
    (9m) => {
        8
    };

    (1p) => {
        9
    };
    (2p) => {
        10
    };
    (3p) => {
        11
    };
    (4p) => {
        12
    };
    (5p) => {
        13
    };
    (6p) => {
        14
    };
    (7p) => {
        15
    };
    (8p) => {
        16
    };
    (9p) => {
        17
    };

    (1s) => {
        18
    };
    (2s) => {
        19
    };
    (3s) => {
        20
    };
    (4s) => {
        21
    };
    (5s) => {
        22
    };
    (6s) => {
        23
    };
    (7s) => {
        24
    };
    (8s) => {
        25
    };
    (9s) => {
        26
    };

    (E) => {
        27
    };
    (S) => {
        28
    };
    (W) => {
        29
    };
    (N) => {
        30
    };
    (P) => {
        31
    };
    (F) => {
        32
    };
    (C) => {
        33
    };

    (5mr) => {
        34
    };
    (5pr) => {
        35
    };
    (5sr) => {
        36
    };

    (?) => {
        37
    };

    [$($s:tt),* $(,)?] => {
        [$($crate::tu8!($s)),*]
    };
}

#[macro_export]
macro_rules! tuz {
    ($s:tt) => {
        $crate::tu8!($s) as usize
    };
    [$($s:tt),* $(,)?] => {
        [$($crate::tuz!($s)),*]
    };
}

#[macro_export]
macro_rules! t {
    ($s:tt) => {
        $crate::tile::Tile($crate::tu8!($s))
    };
    [$($s:tt),* $(,)?] => {
        [$($crate::t!($s)),*]
    };
}

#[macro_export]
macro_rules! matches_tu8 {
    ($o:expr, $($s:tt)|* $(|)?) => {
        matches!($o, $($crate::tu8!($s))|*)
    };
}

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
