#![feature(test)]
extern crate test;

use test::Bencher;
use chocodye::{Dye, make_meal, make_menu, SnackList};

#[inline(always)]
fn bench(starting_dye: Dye, final_dye: Dye) {
    let meal = make_meal(starting_dye, final_dye);
    let _menu = make_menu(starting_dye, SnackList::from(meal.as_slice()));
}

// p2: meal using `chocodye::make_meal::Possibility::<2>`
// d: dye distance

#[bench]
fn bench_p2_00_d_00000(bencher: &mut Bencher) { // identity
    bencher.iter(|| bench(Dye::DesertYellow, Dye::DesertYellow))
}

#[bench]
fn bench_p2_00_d_97278(bencher: &mut Bencher) { // furthest
    bencher.iter(|| bench(Dye::SnowWhite, Dye::SootBlack))
}

#[bench]
fn bench_p2_01_d_01262(bencher: &mut Bencher) { // nearest p2
    bencher.iter(|| bench(Dye::CharcoalGrey, Dye::CurrantPurple))
}

#[bench]
fn bench_p2_01_d_46786(bencher: &mut Bencher) { // furthest p2
    bencher.iter(|| bench(Dye::SkyBlue, Dye::CurrantPurple))
}

#[bench]
fn bench_p2_10_d_38494(bencher: &mut Bencher) { // biggest p2
    bencher.iter(|| bench(Dye::InkBlue, Dye::CoeurlYellow))
}
