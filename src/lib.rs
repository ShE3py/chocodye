use std::{array, fmt};
use std::fmt::Formatter;

pub use dye::{ansi_text, Category, Dye};
pub use rgb::{ParseHexError, Rgb};
pub use snack::Snack;

#[cfg(feature = "fluent")]
pub use crate::fluent::{__format_message, FluentBundle, Lang, ParseLangError};

mod dye;
mod rgb;
mod snack;

#[cfg(feature = "fluent")]
mod fluent;

/// Creates a vector of [`Snack`], that when fed to a chocobo, will change its plumage from one [`Dye`] to another.
///
/// The current implementation is a [brute-force search](https://en.wikipedia.org/wiki/Brute-force_search);
/// it tries all six snacks and takes the one that brings it closest to the desired dye, repeating until this dye is reached.
/// Despite its name, this algorithm is quite fast, as there aren't that much possibilities.
///
/// # Examples
///
/// ```
/// use chocodye::{Dye, make_meal, Snack};
///
/// assert_eq!(Dye::SalmonPink.distance(Dye::RosePink), Dye::EPSILON);
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
        struct Possibility {
            snack: Snack,
            next_color: Rgb,
            next_distance: u32
        }

        let make_possibility = |snack: Snack| snack.alter(current_color).map(|next_color| Possibility { snack, next_color, next_distance: next_color.distance(final_color) });

        let mut possibilities: Vec<_> = Snack::VALUES.iter().copied().filter_map(make_possibility).collect();
        possibilities.sort_unstable_by_key(|possibility| possibility.next_distance);

        let best_choice = possibilities.first().unwrap();

        if current_distance < best_choice.next_distance {
            break;
        }
        else {
            meal.push(best_choice.snack);
            current_color = best_choice.next_color;
            current_distance = best_choice.next_distance;
        }
    }

    meal
}

/// An unsorted list of [`Snack`], can be considered an `EnumMap<Snack, u8>`.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct SnackList(u64);

impl SnackList {
    /// Creates a new, empty `SnackList`.
    pub const fn new() -> SnackList {
        SnackList(0)
    }

    /// Returns how many times a [`Snack`] is contained within `self`.
    pub const fn get(&self, snack: Snack) -> u8 {
        ((self.0 >> (8 * snack as usize)) & 0xFF) as u8
    }

    /// Sets how many times a [`Snack`] is contained within `self`.
    pub fn set(&mut self, snack: Snack, value: u8) {
        self.0 = (self.0 & !(0xFFu64 << (8 * snack as usize))) | (value as u64) << (8 * snack as usize);
    }

    /// Adds *n* [`Snack`] to `self`.
    pub fn add(&mut self, snack: Snack, n: u8) {
        self.set(snack, self.get(snack) + n);
    }
}

impl From<&[Snack]> for SnackList {
    /// Creates a new [`SnackList`] from a slice of [`Snack`].
    fn from(snacks: &[Snack]) -> SnackList {
        let mut sl = SnackList::new();

        for snack in snacks {
            sl.add(*snack, 1);
        }

        sl
    }
}

impl From<SnackList> for [(Snack, u8); 6] {
    fn from(value: SnackList) -> [(Snack, u8); 6] {
        [
            (Snack::Apple,     ((value.0      ) & 0xFF) as u8),
            (Snack::Pear,      ((value.0 >>  8) & 0xFF) as u8),
            (Snack::Berries,   ((value.0 >> 16) & 0xFF) as u8),
            (Snack::Plum,      ((value.0 >> 24) & 0xFF) as u8),
            (Snack::Fruit,     ((value.0 >> 32) & 0xFF) as u8),
            (Snack::Pineapple, ((value.0 >> 40) & 0xFF) as u8)
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
pub fn make_menu(starting_dye: Dye, snacks: SnackList) -> Vec<(Snack, u8)> {
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
    fn backtrack(remaining: SnackList, current_color: Rgb, menu: Vec<(Snack, u8)>) -> Vec<(Snack, u8)> {
        let mut menus = Vec::with_capacity(Snack::VALUES.len());

        // for each snack, try putting the maximum of them so that the color wouldn't overflow
        for (snack, count) in remaining {
            /// Returns the largest number `q` such that `0 <= c + qd <= 255`.
            fn q(c: u8, d: i8) -> u8 {
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
                    // <=>     q  >= -c / d
                    // <=>     q   = ceil(-c / d)
                    // <=>     q   = floor((-c - d - 1) / d)

                    ((-(c as i16) - (d as i16) - 1) / (d as i16)) as u8
                }
            }

            /// Returns the largest number `q` such that `0 <= c + qd <= 255` for `c := { r, g, b }`.
            #[allow(non_snake_case)]
            fn Q(c: Rgb, s: Snack) -> u8 {
                let qr = q(c.r, s.effect().0);
                let qg = q(c.g, s.effect().1);
                let qb = q(c.b, s.effect().2);

                qr.min(qg).min(qb)
            }

            if count > 0 {
                // `q` considering all components and the remaining count
                let n = Q(current_color, snack).min(count);

                if n > 0 {
                    // try backtracking with `n` less `snack` snacks
                    let mut new_map = remaining;
                    new_map.set(snack, count - n);

                    let new_color = Rgb {
                        r: ((current_color.r as i16) + (n as i16) * (snack.effect().0 as i16)) as u8,
                        g: ((current_color.g as i16) + (n as i16) * (snack.effect().1 as i16)) as u8,
                        b: ((current_color.b as i16) + (n as i16) * (snack.effect().2 as i16)) as u8
                    };

                    let mut new_menu = menu.clone();
                    new_menu.push((snack, n));

                    menus.push(backtrack(new_map, new_color, new_menu));
                }
            }
        }

        menus.sort_unstable_by_key(|menu| menu.len());
        menus.into_iter().next().unwrap_or(menu)
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
            assert_eq!(list.0, 0);

            list.set(Snack::Pear, 210);
            assert_ne!(list.0, 0);
            assert_eq!(list.get(Snack::Pear), 210);

            list.set(Snack::Pear, 0);
            assert_eq!(list.0, 0);
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
        }

        #[test]
        fn meals_are_ok() {
            fn assert_meal(starting_dye: Dye, final_dye: Dye) {
                let meal = make_meal(starting_dye, final_dye);

                let mut rgb = starting_dye.color();
                for snack in meal {
                    rgb = snack.alter(rgb).unwrap();
                }

                let dye = Dye::try_from(rgb).unwrap_or_else(identity);

                assert_eq!(dye, final_dye, "{:?} {:?} -> {:?} {:?}, got {:?} {:?} instead", starting_dye, starting_dye.color(), final_dye, final_dye.color(), Dye::try_from(rgb), rgb);
            }

            assert_meal(Dye::DesertYellow, Dye::RoyalBlue);
            assert_meal(Dye::DeepwoodGreen, Dye::RoyalBlue);
            assert_meal(Dye::OthardBlue, Dye::RoyalBlue);
            assert_meal(Dye::MesaRed, Dye::MesaRed);
        }

        #[test]
        fn menus_are_ok() {
            fn assert_menu(starting_dye: Dye, final_dye: Dye) {
                let meal = make_meal(starting_dye, final_dye);
                let menu = make_menu(starting_dye, meal.as_slice().into());

                println!("{:?}", menu);

                let mut rgb = starting_dye.color();
                for (snack, count) in menu {
                    for i in 0..count {
                        rgb = match snack.alter(rgb) {
                            Some(rgb) => rgb,
                            None => panic!("integer overflow on ({:?} {:?}).alter({:?}) (i = {}/{})", snack, snack.effect(), rgb, i, count - 1)
                        }
                    }
                }

                let dye = Dye::try_from(rgb).unwrap_or_else(identity);

                assert_eq!(dye, final_dye, "{:?} {:?} -> {:?} {:?}, got {:?} {:?} instead", starting_dye, starting_dye.color(), final_dye, final_dye.color(), Dye::try_from(rgb), rgb);
            }

            assert_menu(Dye::DesertYellow, Dye::RoyalBlue);
            assert_menu(Dye::DeepwoodGreen, Dye::RoyalBlue);
            assert_menu(Dye::OthardBlue, Dye::RoyalBlue);
            assert_menu(Dye::MesaRed, Dye::MesaRed);
        }
    }
}
