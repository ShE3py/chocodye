use chocodye::{Dye, Lang};

fn  main() {
    let bundles: Vec<_> = Lang::values().iter().map(|lang| lang.into_bundle()).collect();

    for dye in &[Dye::GoobbueGrey, Dye::RolanberryRed, Dye::LoamBrown, Dye::AdamantoiseGreen, Dye::CelesteGreen, Dye::LotusPink] {
        for bundle in &bundles {
            println!("{}", dye.ansi_full_name(bundle));
        }
    }
}
