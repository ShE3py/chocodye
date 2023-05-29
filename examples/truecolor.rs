use chocodye::{Dye, Lang};

fn  main() {
    let bundles: Vec<_> = Lang::values().iter().map(|lang| lang.into_bundle()).collect();

    for bundle in &bundles {
        println!("{}", Dye::AdamantoiseGreen.color_name(bundle));
    }
}
