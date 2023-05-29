use std::collections::HashMap;
pub use dye::{ansi_text, Category, Dye};
pub use rgb::{ParseHexError, Rgb};
pub use snack::Snack;

#[cfg(feature = "fluent")]
pub use crate::fluent::{FluentBundle, Lang};
#[cfg(feature = "fluent")]
pub(crate) use crate::fluent::__format_message;

mod dye;
mod rgb;
mod snack;

#[cfg(feature = "fluent")]
mod fluent;

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

        let mut possibilities: Vec<_> = Snack::values().iter().copied().filter_map(make_possibility).collect();
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

pub fn make_menu(starting_dye: Dye, snacks: &Vec<Snack>) -> Vec<(Snack, u8)> {
    fn backtrack(remaining: HashMap<Snack, u32>, current_color: Rgb, menu: Vec<(Snack, u8)>) -> Vec<(Snack, u8)> {
        let mut menus = Vec::new();

        // for each snack, try putting the maximum of them so that the color wouldn't overflow
        for (snack, count) in &remaining {
            let snack = *snack;
            let count = *count;

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

            // `q` considering all components and the remaining count
            let n = Q(current_color, snack).min(count.try_into().unwrap_or(u8::MAX));

            if n > 0 {
                let new_count = count - (n as u32);

                let mut new_map = remaining.clone();
                if new_count == 0 {
                    new_map.remove(&snack);
                }
                else {
                    new_map.insert(snack, new_count);
                }

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

        menus.sort_unstable_by_key(|menu| menu.len());
        menus.into_iter().next().unwrap_or(menu)
    }

    let mut snack_count = HashMap::new();

    for snack in snacks {
        *(snack_count.entry(*snack).or_insert(0)) += 1;
    }

    backtrack(snack_count, starting_dye.color(), Vec::new())
}

#[cfg(test)]
mod lib {
    mod test {
        use std::convert::identity;

        use super::super::*;

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
                let menu = make_menu(starting_dye, &meal);

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
