#![cfg(feature = "truecolor")]

use std::env;

use crate::Rgb;

fn is_supported() -> bool {
    env::var("COLORTERM").ok().is_some_and(|s| s == "truecolor" || s == "24bit")
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
/// env::remove_var("COLORTERM");
/// assert_eq!(ansi_text(Rgb::RED, "hello world!"), "hello world!");
///
/// env::set_var("COLORTERM", "truecolor");
/// assert_eq!(ansi_text(Rgb::RED, "hello world!"), "\x1B[48;2;255;0;0m\x1B[38;2;255;255;255mhello world!\x1B[0m");
/// //                                                         ^^^^^^^           ^^^^^^^^^^^ ^^^^^^^^^^^^
/// //                                                        background          foreground     text
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "truecolor")))]
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
