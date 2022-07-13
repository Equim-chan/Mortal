//! Updated at 2022/07/13 20:55 JST
//!
//! Source:
//! * <https://tenhou.net/ranking.html>
//! * <http://otokomyouri.com/Ranking/Ranking.aspx>

use boomphf::hashmap::BoomHashMap;
use flate2::read::GzDecoder;
use once_cell::sync::Lazy;

use std::io::prelude::*;

pub static TOP300_2K_GAMES: Lazy<BoomHashMap<String, ()>> = Lazy::new(|| {
    let mut gz = GzDecoder::new(&include_bytes!("data/top300_2k_games.txt.gz")[..]);
    let mut raw = String::new();
    gz.read_to_string(&mut raw).unwrap();

    let names: Vec<_> = raw.trim().lines().map(|s| s.to_owned()).collect();
    let size = names.len();
    assert_eq!(size, 300);

    BoomHashMap::new(names, vec![(); size])
});

pub const TENHOUI: &[&str] = &[
    "じょにおん！！",
    "わっしょい君2号",
    "火時計を押せ！",
    "いばらぎ",
    "yoteru",
    "CLS",
    "藤井聡ふと",
    "右折するひつじ",
    "お知らせ",
    "gousi",
    "おかもと",
    "トトリ先生19歳",
    "ウルトラ立直",
    "就活生@川村軍団",
    "かにマジン",
    "コーラ下さい",
    "タケオしゃん",
    "太くないお",
    "すずめクレイジー",
    "独歩",
    "（≧▽≦）",
    "ASAPIN",
];
