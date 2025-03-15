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

use std::{array, fmt};
use std::convert::identity;
use std::fmt::Formatter;
use std::num::NonZeroU64;

pub use dye::{Category, Dye};
pub use rgb::{ParseHexError, Rgb};
pub use snack::Snack;

#[cfg(feature = "fluent")]
pub use crate::fluent::{FluentBundle, Lang, ParseLangError};
#[cfg(feature = "fluent")]
#[doc(hidden)]
pub use crate::fluent::__format_message;
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
                
                USED_PAIRS.into_iter().filter_map(move |(s, t)| Self::from([s, t], current_color, final_color))
            }
            
            fn get(current_color: Rgb, final_color: Rgb) -> Possibility<2> {
                Self::iter(current_color, final_color).min_by_key(|p| p.next_distance).unwrap()
            }
        }
        
        macro_rules! try_possibilities {
            ($N:literal, $($M:literal),*) => { #[allow(clippy::redundant_else)] {
                let best_choice = Possibility::<$N>::get(current_color, final_color);
                
                if current_distance < best_choice.next_distance {
                    let current_dye = Dye::try_from(current_color).unwrap_or_else(identity);
                    
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

/// An unsorted list of [`Snack`], can be considered an `EnumMap<Snack, u8>`.
///
/// This struct is stored as a [`NonZeroU64`], enabling some memory layout optimization:
///
/// ```
/// use chocodye::SnackList;
/// use std::mem::size_of;
///
/// assert_eq!(size_of::<Option<SnackList>>(), size_of::<u64>());
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SnackList(NonZeroU64);

impl SnackList {
    /// Creates a new, empty `SnackList`.
    #[must_use]
    #[inline]
    pub const fn new() -> SnackList {
        // SAFETY: `1 << 63` is not zero.
        SnackList(unsafe { NonZeroU64::new_unchecked(1 << 63) })
    }

    /// Returns how many times a [`Snack`] is contained within `self`.
    #[must_use]
    #[inline]
    pub const fn get(&self, snack: Snack) -> u8 {
        ((self.0.get() >> (8 * snack as usize)) & 0xFF) as u8
    }

    /// Sets how many times a [`Snack`] is contained within `self`.
    #[inline]
    pub fn set(&mut self, snack: Snack, value: u8) {
        // SAFETY: both `self.0` and `!(0xFFu64 << (8 * snack as usize))` have their msb set to `1`, thus
        // making the result's msb to `1`.
        self.0 = unsafe { NonZeroU64::new_unchecked(
            (self.0.get() & !(0xFF_u64 << (8 * snack as usize))) | ((value as u64) << (8 * snack as usize))
        ) };
    }

    /// Adds *n* [`Snack`] to `self`.
    #[inline]
    pub fn add(&mut self, snack: Snack, n: u8) {
        self.set(snack, self.get(snack) + n);
    }
    
    ///  Returns `true` if `self` has no snacks.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.get() == SnackList::new().0.get()
    }
    
    /// Returns how many snacks are contained within `self`.
    #[must_use]
    pub fn sum(&self) -> u64 {
        let mut me = self.0.get();
        
        let mut acc = 0;
        for _ in 0..7 {
            acc += me & 0xFF;
            me >>= 8;
        }
        
        acc
    }
    
    /// Returns how many kinds of snack are contained within `self`.
    #[must_use]
    pub fn kinds(&self) -> u8 {
        let mut me = self.0.get();
        
        let mut count = 0;
        for _ in 0..7 {
            if (me & 0xFF) != 0 {
                count += 1;
            }
            
            me >>= 8;
        }
        
        count
    }
}

impl From<&[Snack]> for SnackList {
    /// Creates a new [`SnackList`] from a slice of [`Snack`].
    fn from(snacks: &[Snack]) -> SnackList {
        let mut sl = SnackList::new().0.get();

        for snack in snacks {
            sl += 1 << (8 * *snack as usize);
        }

        SnackList(NonZeroU64::new(sl).expect("integer overflow"))
    }
}

impl From<SnackList> for [(Snack, u8); 6] {
    fn from(value: SnackList) -> [(Snack, u8); 6] {
        [
            (Snack::Apple,     ((value.0.get()      ) & 0xFF) as u8),
            (Snack::Pear,      ((value.0.get() >>  8) & 0xFF) as u8),
            (Snack::Berries,   ((value.0.get() >> 16) & 0xFF) as u8),
            (Snack::Plum,      ((value.0.get() >> 24) & 0xFF) as u8),
            (Snack::Fruit,     ((value.0.get() >> 32) & 0xFF) as u8),
            (Snack::Pineapple, ((value.0.get() >> 40) & 0xFF) as u8)
        ]
    }
}

impl IntoIterator for SnackList {
    type Item = (Snack, u8);
    type IntoIter = array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        <SnackList as Into<[Self::Item; 6]>>::into(self).into_iter()
    }
}

impl Default for SnackList {
    /// Creates a new, empty `SnackList`.
    #[inline]
    fn default() -> SnackList {
        SnackList::new()
    }
}

impl fmt::Debug for SnackList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut dm = f.debug_map();

        for snack in Snack::VALUES {
            dm.entry(&snack, &self.get(snack));
        }

        dm.finish()
    }
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
    #[allow(clippy::cast_possible_truncation)]
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
        fn snacklist_get_set() {
            let mut list = SnackList::new();
            assert_eq!(list.0.get(), 1 << 63);
            assert!(list.is_empty());
            assert_eq!(list.sum(), 0);
            assert_eq!(list.kinds(), 0);

            list.set(Snack::Pear, 210);
            list.add(Snack::Pear, 12);
            assert_ne!(list.0.get(), 1 << 63);
            assert_eq!(list.get(Snack::Pear), 222);
            assert!(!list.is_empty());
            assert_eq!(list.sum(), 222);
            assert_eq!(list.kinds(), 1);

            list.set(Snack::Pear, 0);
            assert_eq!(list.0.get(), 1 << 63);
            assert!(list.is_empty());
            assert_eq!(list.sum(), 0);
            assert_eq!(list.kinds(), 0);
        }

        #[test]
        fn snacklist_into_array() {
            let mut list = SnackList::new();
            list.set(Snack::Pear, 1);
            list.set(Snack::Pineapple, 2);
            list.set(Snack::Fruit, 3);
            list.set(Snack::Plum, 4);
            list.set(Snack::Berries, 5);
            list.set(Snack::Apple, 6);

            assert_eq!(<SnackList as Into<[(Snack, u8); 6]>>::into(list), [
                (Snack::Apple, 6),
                (Snack::Pear, 1),
                (Snack::Berries, 5),
                (Snack::Plum, 4),
                (Snack::Fruit, 3),
                (Snack::Pineapple, 2)
            ]);
            
            assert!(!list.is_empty());
            assert_eq!(list.sum(), 21);
            assert_eq!(list.kinds(), 6);
            
            // see safety note of `SnackList::set`
            for snack in Snack::VALUES {
                assert!((snack as u8) < 8);
            }
        }
        
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
