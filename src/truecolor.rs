#![cfg(feature = "truecolor")]

use std::env;

use crate::Rgb;

fn find_subsequence<T: PartialEq>(haystack: &[T], needle: &[T]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn is_supported() -> bool {
    #[cfg(any(unix, target_os = "wasi"))]
    {
        #[cfg(unix)]
        use std::os::unix::ffi::OsStrExt;

        #[cfg(target_os = "wasi")]
        use std::os::wasi::ffi::OsStrExt;

        if let Some(var) = env::var_os("COLORTERM") {
            let bytes = var.as_bytes();

            find_subsequence(bytes, b"truecolor").is_some() || find_subsequence(bytes, b"24bit").is_some()
        } else {
            false
        }
    }

    #[cfg(not(any(unix, target_os = "wasi")))]
    {
        env::var("COLORTERM").map(|val| val.contains("truecolor") || val.contains("24bit")).unwrap_or(false)
    }
}

/// Changes the background color of a string using three [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit)
/// if the terminal support [truecolors](https://en.wikipedia.org/wiki/Color_depth#True_color_(24-bit)).
///
/// If the environment variable `COLORTERM` is not defined or contains neither `truecolor` nor `24bit`, the string parameter
/// is returned as is.
///
/// This function also changes the foreground color according to the specified background color in order to ensure that the
/// text is visible.
///
/// # Examples
///
/// ```
/// use chocodye::{Rgb, ansi_text};
/// use std::env;
///
/// env::set_var("COLORTERM", "256");
/// assert_eq!(ansi_text(Rgb::RED, "hello world!"), "hello world!");
///
/// env::set_var("COLORTERM", "truecolor");
/// assert_eq!(ansi_text(Rgb::RED, "hello world!"), "\x1B[48;2;255;0;0m\x1B[38;2;255;255;255mhello world!\x1B[0m");
/// //                                                         ^^^^^^^           ^^^^^^^^^^^ ^^^^^^^^^^^^
/// //                                                        background          foreground     text
/// ```
#[cfg_attr(docrs, doc(cfg(feature = "truecolor")))]
pub fn ansi_text(bg: Rgb, s: &str) -> String {
    if !is_supported() {
        s.to_owned()
    }
    else {
        let fg = {
            let d = bg.distance(Rgb::WHITE);

            const LIMIT: u32 = Rgb::gray(127).distance(Rgb::WHITE);

            if d >= LIMIT {
                Rgb::WHITE
            } else {
                Rgb::BLACK
            }
        };

        format!("\x1B[48;2;{};{};{}m\x1B[38;2;{};{};{}m{}\x1B[0m",
           bg.r, bg.g, bg.b,
           fg.r, fg.g, fg.b,
           s
        )
    }
}
