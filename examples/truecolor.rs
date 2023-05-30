use std::env;
use std::process::exit;
use std::str::FromStr;

use chocodye::{ansi_text, Category, Lang};

#[cfg(unix)]
fn get_term_width() -> Option<u16> {
    // https://man7.org/linux/man-pages/man2/ioctl_tty.2.html

    use std::io;
    use std::mem::MaybeUninit;
    use libc::{c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};

    #[repr(C)]
    struct TermSize {
        row: c_ushort,
        col: c_ushort,
        x: c_ushort,
        y: c_ushort
    }

    let mut size = MaybeUninit::<TermSize>::zeroed();

    unsafe {
        let ret = ioctl(STDOUT_FILENO, TIOCGWINSZ, size.as_mut_ptr());

        if ret == 0 {
            Some(size.assume_init().col.try_into().unwrap_or(u16::MAX))
        }
        else {
            eprintln!("`ioctl()`: {}", io::Error::last_os_error());

            None
        }
    }
}

#[cfg(not(unix))]
fn get_term_width() -> Option<u16> {
    None
}

fn  main() {
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
            eprintln!("cargo run --example truecolor --features=fluent -- en");
            exit(1);
        }
    };

    let term_width = get_term_width().unwrap_or(u16::MAX) as u32;

    const TAB_WIDTH: u32 = 8;
    const TABS: u32 = 3;

    const BASE_INDENT: u32 = TABS * TAB_WIDTH;

    println!();

    for category in Category::values() {
        let mut dyes: Vec<_> = category.dyes().to_vec();
        dyes.sort_unstable_by_key(|dye| 255 - dye.luma());

        let category_full_name = category.full_name(&bundle);
        let tabs = "\t".repeat(((BASE_INDENT - category_full_name.len() as u32 + (TAB_WIDTH - 1)) / TAB_WIDTH) as usize);
        print!("{}{tabs}", ansi_text(category.color(), category_full_name.as_ref()));

        let mut current_width = BASE_INDENT;
        let carriage = ansi_text(category.color(), &" ".repeat(category_full_name.len()));

        for dye in dyes {
            let color_name = dye.color_name(&bundle);
            let char_count = (color_name.chars().count() + 1) as u32;

            if (current_width + char_count) > (term_width as u32) {
                println!();
                print!("{}{tabs}", carriage);
                current_width = BASE_INDENT;
            }

            print!("{} ", dye.ansi_color_name(&bundle));
            current_width += char_count;
        }

        println!();
    }

    println!();
}
