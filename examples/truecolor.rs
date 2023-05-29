use chocodye::{ansi_text, Dye, Lang};

fn  main() {
    let bundles: Vec<_> = Lang::values().iter().map(|lang| lang.into_bundle()).collect();

    for dye in &[Dye::GoobbueGrey, Dye::RolanberryRed, Dye::LoamBrown, Dye::AdamantoiseGreen, Dye::RoyalBlue, Dye::LotusPink] {
        for bundle in &bundles {
            println!("{} {}", dye.ansi_full_name(bundle), ansi_text(dye.category().color(), &format!("({})", dye.category().full_name(bundle))));
        }
    }
}
