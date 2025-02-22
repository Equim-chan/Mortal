use riichi::algo::agari::{self, AgariCalculator};
use riichi::algo::shanten;
use riichi::algo::sp::{InitState, SPCalculator};
use riichi::hand::hand;
use riichi::state::PlayerState;
use riichi::{t, tu8};
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn shanten(c: &mut Criterion) {
    shanten::ensure_init();
    let tehai = hand("2344456m 14p 127s 2z 7p").unwrap();
    c.bench_function("shanten", |b| {
        b.iter(|| {
            let tehai = black_box(tehai);
            black_box(shanten::calc_all(&tehai, 4));
        });
    });
}

fn agari(c: &mut Criterion) {
    agari::ensure_init();
    let tehai = hand("111m 9m 9m").unwrap();
    c.bench_function("agari", |b| {
        b.iter(|| {
            let tehai = black_box(tehai);
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
            black_box(calc.search_yakus().unwrap());
        });
    });
}

fn sp(c: &mut Criterion) {
    let calc = SPCalculator {
        tehai_len_div3: 4,
        chis: &[],
        pons: &[],
        minkans: &[],
        ankans: &[],
        bakaze: tu8!(E),
        jikaze: tu8!(E),
        prefer_riichi: true,
        is_menzen: true,
        num_doras_in_fuuro: 0,
        dora_indicators: &[t!(6m)],
        calc_double_riichi: false,
        calc_haitei: false,
        sort_result: true,
        maximize_win_prob: true,
        calc_tegawari: true,
        calc_shanten_down: true,
    };
    let tehai = hand("3667m 23489p 34688s").unwrap();
    let mut tiles_seen = tehai;
    for ind in calc.dora_indicators {
        tiles_seen[ind.deaka().as_usize()] += 1;
    }
    let can_discard = true;
    let cur_shanten = shanten::calc_all(&tehai, calc.tehai_len_div3);
    let tsumos_left = 12;
    let init_state = InitState {
        tehai,
        akas_in_hand: [false; 3],
        tiles_seen,
        akas_seen: [false; 3],
    };
    c.bench_function(&format!("sp {cur_shanten} shanten"), |b| {
        b.iter(|| {
            let state = black_box(init_state.clone());
            let candidates = black_box(&calc)
                .calc(state, can_discard, tsumos_left, cur_shanten)
                .unwrap();
            black_box(candidates);
        });
    });

    // ---

    let calc = SPCalculator {
        tehai_len_div3: 4,
        chis: &[],
        pons: &[],
        minkans: &[],
        ankans: &[],
        bakaze: tu8!(E),
        jikaze: tu8!(E),
        prefer_riichi: true,
        is_menzen: true,
        num_doras_in_fuuro: 0,
        dora_indicators: &[t!(6m)],
        calc_double_riichi: false,
        calc_haitei: false,
        sort_result: true,
        maximize_win_prob: true,
        calc_tegawari: true,
        calc_shanten_down: true,
    };
    let tehai = hand("45677m 456778p 248s").unwrap();
    let mut tiles_seen = tehai;
    for ind in calc.dora_indicators {
        tiles_seen[ind.deaka().as_usize()] += 1;
    }
    let can_discard = true;
    let cur_shanten = shanten::calc_all(&tehai, calc.tehai_len_div3);
    let tsumos_left = 12;
    let init_state = InitState {
        tehai,
        akas_in_hand: [false; 3],
        tiles_seen,
        akas_seen: [false; 3],
    };
    c.bench_function(&format!("sp {cur_shanten} shanten"), |b| {
        b.iter(|| {
            let state = black_box(init_state.clone());
            let candidates = black_box(&calc)
                .calc(state, can_discard, tsumos_left, cur_shanten)
                .unwrap();
            black_box(candidates);
        });
    });
}

fn encode_obs(c: &mut Criterion) {
    let log = r#"
        {"type":"start_kyoku","bakaze":"S","dora_marker":"F","kyoku":1,"honba":2,"kyotaku":1,"oya":0,"scores":[32300,18000,22000,26700],"tehais":[["1m","3m","4m","5mr","6m","6m","7m","5p","8p","6s","E","W","N"],["4m","5m","5m","8m","7p","1s","3s","4s","7s","9s","E","E","N"],["1m","2m","3m","3m","7m","9m","6p","9p","1s","4s","5sr","8s","C"],["8m","8m","3p","4p","4p","5p","8p","1s","6s","9s","9s","S","F"]]}
        {"type":"tsumo","actor":0,"pai":"C"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"N"}
        {"type":"dahai","actor":1,"pai":"8m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"5s"}
        {"type":"dahai","actor":2,"pai":"1s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"7p"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"7s"}
        {"type":"dahai","actor":0,"pai":"N","tsumogiri":false}
        {"type":"pon","actor":1,"target":0,"pai":"N","consumed":["N","N"]}
        {"type":"dahai","actor":1,"pai":"7p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"6p"}
        {"type":"dahai","actor":2,"pai":"9p","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2m"}
        {"type":"dahai","actor":3,"pai":"F","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"4p"}
        {"type":"dahai","actor":0,"pai":"8p","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"P"}
        {"type":"dahai","actor":1,"pai":"4m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"7s"}
        {"type":"dahai","actor":2,"pai":"5s","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"2p"}
        {"type":"dahai","actor":3,"pai":"S","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"2p"}
        {"type":"dahai","actor":0,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"2s"}
        {"type":"dahai","actor":1,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"3s"}
        {"type":"dahai","actor":2,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6m"}
        {"type":"dahai","actor":3,"pai":"2m","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"W"}
        {"type":"dahai","actor":0,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"3p"}
        {"type":"dahai","actor":1,"pai":"5m","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"E"}
        {"type":"dahai","actor":2,"pai":"E","tsumogiri":true}
        {"type":"pon","actor":1,"target":2,"pai":"E","consumed":["E","E"]}
        {"type":"dahai","actor":1,"pai":"3p","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"2p"}
        {"type":"dahai","actor":2,"pai":"2p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2p"}
        {"type":"dahai","actor":3,"pai":"6s","tsumogiri":false}
        {"type":"tsumo","actor":0,"pai":"1m"}
        {"type":"dahai","actor":0,"pai":"E","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"9p"}
        {"type":"dahai","actor":1,"pai":"9p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8p"}
        {"type":"dahai","actor":2,"pai":"8p","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"5s"}
        {"type":"dahai","actor":3,"pai":"5s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"3s"}
        {"type":"dahai","actor":0,"pai":"C","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"C"}
        {"type":"dahai","actor":1,"pai":"P","tsumogiri":false}
        {"type":"tsumo","actor":2,"pai":"S"}
        {"type":"dahai","actor":2,"pai":"S","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"3s"}
        {"type":"dahai","actor":3,"pai":"3s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6p"}
        {"type":"dahai","actor":0,"pai":"3s","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"4m"}
        {"type":"dahai","actor":1,"pai":"4m","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"S"}
        {"type":"dahai","actor":2,"pai":"S","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"1s"}
        {"type":"dahai","actor":3,"pai":"1s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"7m"}
        {"type":"dahai","actor":0,"pai":"6m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"W"}
        {"type":"dahai","actor":1,"pai":"W","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"8s"}
        {"type":"dahai","actor":2,"pai":"3m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"1m"}
        {"type":"dahai","actor":3,"pai":"1m","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"9m"}
        {"type":"dahai","actor":0,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"1p"}
        {"type":"dahai","actor":1,"pai":"1p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"3m"}
        {"type":"dahai","actor":2,"pai":"3m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"2s"}
        {"type":"dahai","actor":3,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"6s"}
        {"type":"dahai","actor":0,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":1,"pai":"9p"}
        {"type":"dahai","actor":1,"pai":"9p","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"9m"}
        {"type":"dahai","actor":2,"pai":"9m","tsumogiri":true}
        {"type":"tsumo","actor":3,"pai":"P"}
        {"type":"dahai","actor":3,"pai":"P","tsumogiri":true}
        {"type":"tsumo","actor":0,"pai":"5p"}
        {"type":"dahai","actor":0,"pai":"5p","tsumogiri":true}
        {"type":"tsumo","actor":1,"pai":"2s"}
        {"type":"dahai","actor":1,"pai":"2s","tsumogiri":true}
        {"type":"tsumo","actor":2,"pai":"5s"}
        {"type":"dahai","actor":2,"pai":"7m","tsumogiri":false}
        {"type":"tsumo","actor":3,"pai":"6s"}
    "#;
    let mut ps = PlayerState::new(3);
    for line in log.trim().split('\n') {
        ps.update(&serde_json::from_str(line).unwrap()).unwrap();
    }

    c.bench_function("encode obs", |b| {
        b.iter(|| {
            let ps = black_box(&ps);
            let result = ps.encode_obs(4, false);
            black_box(result);
        });
    });
}

criterion_group!(algo, shanten, agari, sp);
criterion_group!(state, encode_obs);
criterion_main!(algo, state);
