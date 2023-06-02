use std::{env, io};
use std::io::{BufRead, Write};
use std::process::exit;
use std::str::FromStr;

use chocodye::{Dye, FluentBundle, Lang, make_meal, make_menu, message, SnackList};


fn ask_dye(bundle: &FluentBundle, question: &'static str, default: Option<Dye>) -> io::Result<Dye> {
    let mut buf = String::with_capacity(32);
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();

    let question = message!(bundle, question);

    let dye = loop {
        stdout.write_all(question.as_bytes())?;
        stdout.write_all(b" ")?;
        stdout.flush()?;

        buf.clear();
        stdin.read_line(&mut buf)?;

        let trimmed = buf.trim();

        if let Some(dye) = Dye::from_str(bundle, trimmed) {
            break dye;
        }

        if trimmed.is_empty() {
            if let Some(default) = default {
                break default;
            }
        }
    };

    // the final space is for overwritting in case of `ss` -> `ß` conversion
    //                    v
    println!("\x1B[1A{} {} ", question, dye.ansi_color_name(bundle));
    Ok(dye)
}

fn main() -> io::Result<()> {
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
            eprintln!("cargo run --example menu -- en");
            exit(1);
        }
    };

    let starting_dye = ask_dye(&bundle, "starting-color-input", Some(Dye::DEFAULT_CHOCOBO_COLOR))?;
    let final_dye = ask_dye(&bundle, "final-color-input", None)?;

    println!();

    let meal = make_meal(starting_dye, final_dye);
    let snacks = SnackList::from(meal.as_slice());

    println!("{}", message!(&bundle, "required-fruits"));
    for (snack, count) in snacks.into_iter().filter(|(_, count)| *count > 0) {
        println!("– {}", snack.quantified_name(&bundle, count as u32));
    }
    println!();

    let menu = make_menu(starting_dye, snacks);
    println!("{}", message!(&bundle, "feed-order"));
    for (snack, count) in menu {
        println!("– {}", snack.quantified_name(&bundle, count as u32));
    }

    Ok(())
}
