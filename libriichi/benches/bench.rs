use riichi::algo::agari;
use riichi::algo::shanten;
use riichi::hand::hand;
use riichi::tu8;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    agari::ensure_init();
    shanten::ensure_init();

    let tehai = hand("111m 9m 9m").unwrap();
    c.bench_function("shanten", |b| {
        b.iter(|| {
            let tehai = black_box(tehai);
            let calc = agari::AgariCalculator {
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
            calc.search_yaku(false).unwrap();
        })
    });

    let tehai = hand("2344456m 14p 127s 2z 7p").unwrap();
    c.bench_function("agari", |b| {
        b.iter(|| {
            let tehai = black_box(tehai);
            shanten::calc_all(&tehai, 4);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

/*
AMD Ryzen 5 3600 on Windows 10
rustc 1.57.0 (f1edd0429 2021-11-29)
-C target-cpu=native -C link-arg=fuse-ld=lld

shanten                     time:   [202.38 ns 202.67 ns 203.00 ns]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe

agari                       time:   [88.025 ns 88.238 ns 88.444 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
*/
