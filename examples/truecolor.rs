use std::env;
use std::process::exit;
use std::str::FromStr;

use unic_langid::langid;

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

#[cfg(windows)]
fn get_term_width() -> Option<u16> {
    use std::io;
    use std::mem::MaybeUninit;
    use windows_sys::Win32::System::Console::{CONSOLE_SCREEN_BUFFER_INFO, GetConsoleScreenBufferInfo, GetStdHandle, STD_OUTPUT_HANDLE};
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

    let mut size = MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::zeroed();

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);

        if handle == INVALID_HANDLE_VALUE {
            eprintln!("`GetStdHandle()`: {}", io::Error::last_os_error());

            return None;
        }

        if handle.is_null() {
            return None;
        }

        let ret = GetConsoleScreenBufferInfo(handle, size.as_mut_ptr());

        if ret != 0 {
            let width = size.assume_init().dwSize.X;

            match width.try_into() {
                Ok(w) => Some(w),
                Err(e) => {
                    eprintln!("`GetConsoleScreenBufferInfo()`: `dwSize.X`: {}", e);

                    None
                }
            }
        }
        else {
            eprintln!("`GetConsoleScreenBufferInfo()`: {}", io::Error::last_os_error());

            None
        }
    }
}

#[cfg(not(any(unix, windows)))]
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
            eprintln!("cargo run --example truecolor -- en");
            exit(1);
        }
    };

    let term_width = get_term_width().unwrap_or(u16::MAX) as u32;

    const TAB_WIDTH: u32 = 8;
    const TABS: u32 = 3;

    const BASE_INDENT: u32 = TABS * TAB_WIDTH;

    let char_weight = match bundle.locales.iter().next() {
        Some(locale) if *locale == langid!("jp") => 2,
        _ => 1
    };

    const PAD_CAT: u32 = BASE_INDENT - 4;

    println!();

    for category in Category::VALUES {
        let mut dyes: Vec<_> = category.dyes().to_vec();
        dyes.sort_unstable_by_key(|dye| 255 - dye.luma());

        let category_full_name = category.full_name(&bundle);
        let colored_category_name = ansi_text(category.color(), category_full_name);

        print!("{:>pad$}\t", colored_category_name, pad = PAD_CAT as usize + (colored_category_name.len() - category_full_name.len()));

        let mut current_width = BASE_INDENT;
        let carriage = format!("{}{}", " ".repeat(PAD_CAT as usize - category_full_name.len()), ansi_text(category.color(), &" ".repeat(category_full_name.len())));

        for dye in dyes {
            let color_name = dye.color_name(&bundle);
            let char_count = (color_name.chars().count() + 1) as u32;

            if (current_width + char_count) > term_width {
                println!();
                print!("{}\t", carriage);
                current_width = BASE_INDENT;
            }

            print!("{} ", dye.ansi_color_name(&bundle));
            current_width += char_count * char_weight;
        }

        println!();
    }

    println!();
}
