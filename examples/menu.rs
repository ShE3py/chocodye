use std::env;
use std::process::exit;
use std::str::FromStr;

use chocodye::{count_snacks, Dye, Lang, make_meal, make_menu, message};

macro_rules! print_rows {
    ($bundle:expr, $iter:expr) => {
        for (snack, count) in $iter {
            println!("{}", message!($bundle, "row", { "content" = message!($bundle, snack.short_name(), { "quantity" = *count }) }));
        }
    };
}

fn main() {
    const STARTING_DYE: Dye = Dye::DEFAULT_CHOCOBO_COLOR;
    const FINAL_DYE: Dye = Dye::RolanberryRed;

    let bundle = match env::args_os().skip(1).next() {
        Some(arg) => {
            match arg.to_str() {
                Some(arg) => match Lang::from_str(arg) {
                    Ok(lang) => lang.into_bundle(),
                    Err(_) => {
                        eprintln!("Unknown language: `{}`.", arg);
                        exit(1);
                    }
                },
                None => {
                    eprintln!("Invalid unicode: `{}`.", arg.to_string_lossy());
                    exit(1);
                }
            }
        },

        None => {
            eprintln!("Please select a language from `en`, `fr`, `de` or `jp`.");
            eprintln!();
            eprintln!("Example:");
            eprintln!("cargo run --example menu --features=fluent -- en");
            exit(1);
        }
    };

    println!("{}", message!(&bundle, "starting-color", { "name" = STARTING_DYE.ansi_color_name(&bundle) }));
    println!("{}", message!(&bundle, "final-color", { "name" = FINAL_DYE.ansi_color_name(&bundle) }));
    println!();

    let meal = make_meal(STARTING_DYE, FINAL_DYE);
    let snacks = count_snacks(&meal);

    println!("{}", message!(&bundle, "required-fruits"));
    print_rows!(&bundle, snacks.iter());
    println!();

    let menu = make_menu(STARTING_DYE, &snacks);
    println!("{}", message!(&bundle, "instructions"));
    print_rows!(&bundle, menu.iter());
}
