//! A Rust library for changing the color of the chocobos' plumage in *Final Fantasy XIV*.
//!
//! # Features
//!
//! - `fluent`: provides access to localized fruit and color names,
//! but only for English, French, German and Japanese.
//!
//! - `truecolor`: enables colored text to be displayed on terminals
//! supporting 24-bit color.
//!
//! These two features are enabled by default.
//!
//! # Examples
//!
//! To print all the dyes:
//!

#![cfg_attr(feature = "fluent", doc = r#"
```no_run
use chocodye::{Dye, Lang};

let bundle = Lang::English.into_bundle();

let mut dyes = Dye::VALUES;
dyes.sort_unstable_by_key(|dye| 255 - dye.luma());

for dye in dyes {
    print!("{} ", dye.ansi_color_name(&bundle));
}

println!();
```
"#)]

#![cfg_attr(not(feature = "fluent"), doc = r#"
```no_run
use chocodye::Dye;

let mut dyes = Dye::VALUES;
dyes.sort_unstable_by_key(|dye| 255 - dye.luma());

println!("{:#?}", dyes);
```
"#)]

//!
//! To print all the dyes by category:
//!

#![cfg_attr(feature = "fluent", doc = r#"
```no_run
use chocodye::{Category, Lang};

let bundle = Lang::English.into_bundle();

for category in Category::VALUES {
    print!("{} -- ", category.ansi_full_name(&bundle));

    for dye in category.dyes() {
        print!("{} ", dye.ansi_color_name(&bundle));
    }

    println!();
}
```
"#)]

#![cfg_attr(not(feature = "fluent"), doc = r#"
```no_run
use chocodye::Category;

for category in Category::VALUES {
    println!("{:?} {:#?}", category, category.dyes());
}
```
"#)]

//!
//! To print a menu:
//!
//! ```
//! use chocodye::{Dye, make_meal, make_menu, SnackList};
//!
//! let meal = make_meal(Dye::SnowWhite, Dye::BoneWhite);
//! let menu = make_menu(Dye::SnowWhite, SnackList::from(meal.as_slice()));
//!
//! println!("{:?}", menu);
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use dye::{Category, Dye};
pub use rgb::{ParseHexError, Rgb};
pub use snack::{Snack, SnackList};

#[cfg(feature = "fluent")]
pub use crate::fluent::{FluentBundle, Lang, ParseLangError};
#[cfg(feature = "fluent")]
#[doc(hidden)]
pub use crate::fluent::{__format_message, __FluentArgs};
#[cfg(feature = "truecolor")]
pub use crate::truecolor::ansi_text;

mod dye;
mod fluent;
mod rgb;
mod snack;
mod truecolor;

/// Creates a vector of [`Snack`], that when fed to a chocobo, will change its plumage from one [`Dye`] to another.
///
/// The current implementation is a [greedy algorithm](https://en.wikipedia.org/wiki/Greedy_algorithm);
/// it tries all six snacks and takes the one that brings it closest to the desired dye, repeating until this dye is reached.
///
/// If adding a single snack can't get it any closer to its goal, it will try with two separate snacks.
/// No meal will need to use more than two snacks in order to get closer to its goal.
///
/// # Examples
///
/// ```
/// use chocodye::{Dye, make_meal, Snack};
///
/// assert_eq!(make_meal(Dye::SalmonPink, Dye::RosePink), [ Snack::Fruit,  Snack::Berries]);
/// assert_eq!(make_meal(Dye::RosePink, Dye::SalmonPink), [-Snack::Fruit, -Snack::Berries]);
/// ```
#[must_use]
pub fn make_meal(starting_dye: Dye, final_dye: Dye) -> Vec<Snack> {
    let mut meal = Vec::new();

    let final_color = final_dye.color();

    let mut current_color = starting_dye.color();
    let mut current_distance = current_color.distance(final_color);

    loop {
        // find the best snack (N = 1) or the best two snacks (N = 2)
        // in order to make `current_color` nearer to `final_color`
        struct Possibility<const N: usize> {
            snacks: [Snack; N],
            next_color: Rgb,
            next_distance: u32
        }

        impl<const N: usize> Possibility<N> {
            fn from(snacks: [Snack; N], current_color: Rgb, final_color: Rgb) -> Option<Possibility<N>> {
                snacks.iter().copied().try_fold(current_color, |current_color, snack| snack.alter(current_color)).map(|next_color| Possibility { snacks,  next_color, next_distance: next_color.distance(final_color) })
            }
        }

        impl Possibility<1> {
            fn iter(current_color: Rgb, final_color: Rgb) -> impl Iterator<Item = Possibility<1>> {
                Snack::VALUES.into_iter().filter_map(move |s| Self::from([s], current_color, final_color))
            }

            fn get(current_color: Rgb, final_color: Rgb) -> Possibility<1> {
                Self::iter(current_color, final_color).min_by_key(|p| p.next_distance).unwrap()
            }
        }

        impl Possibility<2> {
            fn iter(current_color: Rgb, final_color: Rgb) -> impl Iterator<Item = Possibility<2>> {
                use Snack::*;

                // Opposites: (Apple, Plum), (Pear, Fruit), (Berries, Pineapple)

                const _PAIRS: [(Snack, Snack); Snack::VALUES.len() * (Snack::VALUES.len() - 2)] = [
                    (Apple, Pear), (Apple, Berries), (Apple, Fruit), (Apple, Pineapple),
                    (Pear, Apple), (Pear, Berries), (Pear, Plum), (Pear, Pineapple),
                    (Berries, Apple), (Berries, Pear), (Berries, Plum), (Berries, Fruit),
                    (Plum, Pear), (Plum, Berries),  (Plum, Fruit), (Plum, Pineapple),
                    (Fruit, Apple), (Fruit, Berries), (Fruit, Plum), (Fruit, Pineapple),
                    (Pineapple, Apple), (Pineapple, Pear), (Pineapple, Plum), (Pineapple, Fruit)
                ];

                // Only the following pairs are actually used
                const USED_PAIRS: [(Snack, Snack); 5] = [
                    (Apple, Pear), (Apple, Berries), (Pear, Berries), (Plum, Pineapple), (Fruit, Pineapple)
                ];

                USED_PAIRS.into_iter().filter_map(move |st| Self::from(st.into(), current_color, final_color))
            }

            fn get(current_color: Rgb, final_color: Rgb) -> Possibility<2> {
                Self::iter(current_color, final_color).min_by_key(|p| p.next_distance).unwrap()
            }
        }

        macro_rules! try_possibilities {
            ($N:literal, $($M:literal),*) => { #[allow(clippy::redundant_else)] {
                let best_choice = Possibility::<$N>::get(current_color, final_color);

                if current_distance < best_choice.next_distance {
                    let current_dye = Dye::try_from(current_color).unwrap_or_else(|d| d);

                    if current_dye == final_dye {
                        break;
                    }
                    else {
//                      println!("using Possibility<2>! {starting_dye:?} {final_dye:?} {}", starting_dye.distance(final_dye));
                        try_possibilities! { $($M),* }
                    }
                }
                else {
                    meal.extend(best_choice.snacks);
                    current_color = best_choice.next_color;
                    current_distance = best_choice.next_distance;
                }
            }};

            ($N:literal) => {{ try_possibilities! { $N, } }};

            () => {{ unreachable!("Possibility<3>") }};
        }

        // try using one snack, or two if one snack can no longer
        // gets us any closer to the final color
        try_possibilities! { 1, 2 }
    }

    meal
}

/// Reduces the complexity of a meal made with [`make_meal`] while preserving the same change of plumage.
///
/// The current implementation is a [backtracking](https://en.wikipedia.org/wiki/Backtracking) algorithm;
/// for each type of [`Snack`], it tries to remove as many as possible that won't overflow the [color](`Rgb`),
/// and then recurses until there are no more snacks.
/// It then returns the candidate with the fewest groups.
///
/// # Examples
///
/// ```
/// use chocodye::{Dye, make_meal, make_menu, Snack::*, SnackList};
///
/// let meal = make_meal(Dye::BarkBrown, Dye::MesaRed);
/// let menu = make_menu(Dye::BarkBrown, SnackList::from(meal.as_slice()));
///
/// assert_eq!(meal, [Apple, Apple, Apple, Apple, Pear, Apple, Pear, Apple, Pear, Apple]);
/// assert_eq!(menu, [(Apple, 7), (Pear, 3)]);
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn make_menu(starting_dye: Dye, snacks: SnackList) -> Vec<(Snack, u8)> {
    /// Returns the largest number `q` such that `0 <= c + qd <= 255`.
    #[inline]
    const fn max(c: u8, d: i8) -> u8 {
        if d > 0 {
            //     c + qd <= 255
            // <=>     qd <= 255 - c
            // <=>      q <= (255 - c) / d
            // <=>      q  = floor((255 - c) / d)

            (u8::MAX - c) / (d as u8)
        }
        else {
            //     c + qd >= 0
            // <=>     qd >= -c
            // <=>     q  <= -c / d
            // <=>     q   = floor(-c / d)

            (-(c as i16) / (d as i16)) as u8
        }
    }

    /// Returns the largest number `q` such that `0 <= c + qd <= 255` for `c := { r, g, b }`.
    #[allow(non_snake_case)]
    fn Q(c: Rgb, s: Snack) -> u8 {
        let qr = max(c.r, s.effect().0);
        let qg = max(c.g, s.effect().1);
        let qb = max(c.b, s.effect().2);

        qr.min(qg).min(qb)
    }

    /// # Backtracking parameters
    ///
    /// - `remaining`: snacks that have yet to be added.
    /// - `current_color`: the current color after having ate all the `menu` snacks.
    /// - `menu`: snacks that have already been added.
    ///
    /// # Returns
    ///
    /// The smallest menu beginning with `menu` after having removed some snacks in `remaining`.
    ///
    fn backtrack(remaining: SnackList, current_color: Rgb, current_menu: Vec<(Snack, u8)>) -> Vec<(Snack, u8)> {
        let mut menu = Vec::new();

        // for each snack, try putting the maximum of them so that the color wouldn't overflow
        for (snack, count) in remaining {
            // `q` considering all components and the remaining count
            let n = Ord::min(Q(current_color, snack), count);
            if n == 0 {
                continue;
            }

            // try backtracking with `n` less `snack` snacks
            let mut bt_remaning = remaining;
            bt_remaning.set(snack, count - n);

            let bt_color = Rgb {
                r: ((current_color.r as i16) + (n as i16) * (snack.effect().0 as i16)) as u8,
                g: ((current_color.g as i16) + (n as i16) * (snack.effect().1 as i16)) as u8,
                b: ((current_color.b as i16) + (n as i16) * (snack.effect().2 as i16)) as u8
            };

            let mut bt_menu = Vec::with_capacity(current_menu.len() + 1);
            bt_menu.extend_from_slice(&current_menu);
            bt_menu.push((snack, n));

            let bt_result = backtrack(bt_remaning, bt_color, bt_menu);
            if bt_result.len() < menu.len() || menu.is_empty() {
                menu = bt_result;
            }
        }

        if !menu.is_empty() {
            menu
        }
        else {
            debug_assert!(remaining.is_empty(), "remaining {remaining:?} not empty at {current_color:?}");
            current_menu
        }
    }

    backtrack(snacks, starting_dye.color(), Vec::new())
}

#[cfg(test)]
mod lib {
    mod test {
        use std::convert::identity;

        use super::super::*;

        #[test]
        #[cfg_attr(miri, ignore)]
        fn all_is_ok() {
            for src in Dye::VALUES {
                for dst in Dye::VALUES {
                    let meal = make_meal(src, dst);
                    let snacks = SnackList::from(meal.as_slice());

                    let mut rgb = src.color();
                    for snack in meal {
                        rgb = snack.alter(rgb).unwrap();
                    }

                    let dye = Dye::try_from(rgb).unwrap_or_else(identity);
                    assert_eq!(dye, dst, "make_meal({src:?}, {dst:?}) returned {dye:?} (d = {})", dye.distance(dst));

                    let menu = make_menu(src, snacks);

                    let mut rgb = src.color();
                    for (snack, count) in menu.clone() {
                        for i in 0..count {
                            rgb = match snack.alter(rgb) {
                                Some(rgb) => rgb,
                                None => panic!("integer overflow on ({:?} {:?}).alter({:?}) (i = {}/{})", snack, snack.effect(), rgb, i, count - 1)
                            }
                        }
                    }

                    let dye = Dye::try_from(rgb).unwrap_or_else(identity);
                    assert_eq!(dye, dst, "make_menu({src:?}, {dst:?}) returned {dye:?} (d = {}, sl = {snacks:#?}, menu = {menu:#?})", dye.distance(dst));
                }
            }
        }
    }
}
