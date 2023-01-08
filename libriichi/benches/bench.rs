use riichi::algo::agari::{self, AgariCalculator};
use riichi::algo::shanten;
use riichi::hand::hand;
use riichi::tu8;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    agari::ensure_init();
    shanten::ensure_init();

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
            calc.search_yakus().unwrap();
        });
    });

    let tehai = hand("2344456m 14p 127s 2z 7p").unwrap();
    c.bench_function("shanten", |b| {
        b.iter(|| {
            let tehai = black_box(tehai);
            let _ = shanten::calc_all(&tehai, 4);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

/*
AMD Ryzen 5 3600 on Windows 10
rustc 1.60.0 (7737e0b5c 2022-04-04)
-C target-cpu=native -C link-arg=fuse-ld=lld

agari                   time:   [202.10 ns 202.56 ns 203.12 ns]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

shanten                 time:   [87.103 ns 87.209 ns 87.359 ns]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
*/
