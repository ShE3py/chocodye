#[cfg(feature = "fluent")]
use {crate::{FluentBundle, message}, std::borrow::Cow};

use crate::Rgb;

include!(concat!(env!("OUT_DIR"), "/dye.rs"));

impl From<Dye> for Rgb {
    fn from(dye: Dye) -> Rgb {
        dye.color()
    }
}

pub fn ansi_text(bg: Rgb, s: &str) -> String {
    let fg = {
        let d = bg.distance(Rgb::WHITE);

        const LIMIT: u32 = Rgb::gray(127).distance(Rgb::WHITE);

        if d >= LIMIT {
            Rgb::WHITE
        }
        else {
            Rgb::BLACK
        }
    };

    format!("\x1B[48;2;{};{};{}m\x1B[38;2;{};{};{}m{}\x1B[0m",
        bg.r, bg.g, bg.b,
        fg.r, fg.g, fg.b,
        s
    )
}

impl TryFrom<Rgb> for Dye {
    type Error = Dye;

    fn try_from(value: Rgb) -> Result<Dye, Self::Error> {
        let mut iter = Dye::values().iter();

        let mut min = {
            let first = iter.next().unwrap();
            let d = first.color().distance(value);

            if d == 0 {
                return Ok(*first);
            }
            else if d < Dye::EPSILON {
                return Err(*first);
            }

            (d, *first)
        };

        for dye in iter {
            let d = dye.color().distance(value);

            if d < min.0 {
                if d == 0 {
                    return Ok(*dye);
                }
                else if d < Dye::EPSILON {
                    return Err(*dye);
                }
                else {
                    min = (d, *dye);
                }
            }
        }

        Err(min.1)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Dye {
    fn default() -> Dye {
        Dye::DEFAULT_CHOCOBO_COLOR
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn dyes_in_self_category() {
        assert_eq!(Dye::values().len(), Category::VALUES.iter().map(|category| category.dyes().len()).sum());

        for category in Category::VALUES {
            assert!(category.dyes().iter().all(|dye| dye.category() == category));
        }
    }

    #[test]
    pub fn dyes_epsilon() {
        let mut min = Rgb::new(0, 0, 0).distance(Rgb::new(255, 255, 255)) + 1;

        for dye in Dye::values() {
            let mut others = Dye::values().iter().filter(|d| *d != dye);

            let mut epsilon = {
                let other = others.next().unwrap();

                (dye.color().distance(other.color()), *other)
            };

            for other in others {
                let d = other.distance(epsilon.1);

                if d < epsilon.0 {
                    epsilon = (d, *other);
                }
            }

            if epsilon.0 < min {
                min = epsilon.0;
            }
        }

        assert_eq!(min, Dye::EPSILON);
    }
}
