#[cfg(feature = "fluent")]
use crate::{FluentBundle, message};

#[cfg(all(feature = "fluent", feature = "truecolor"))]
use crate::ansi_text;

use crate::Rgb;

include!(concat!(env!("OUT_DIR"), "/dye.rs"));

#[inline(always)]
#[cfg(feature = "fluent")]
fn from_str_impl(bundle: &FluentBundle, s: &str) -> Option<Dye> {
    let s = s.to_lowercase();

    Dye::VALUES.into_iter().find(|dye| dye.color_name(bundle).replace('ß', "ss").to_lowercase() == s)
}

impl From<Dye> for Rgb {
    /// Converts a dye into its color.
    fn from(dye: Dye) -> Rgb {
        dye.color()
    }
}

impl TryFrom<Rgb> for Dye {
    // the `Error` type is further down so that it stays below the function in the generated documentation,
    // so that the reader reads the type description after the fn description

    /// Converts a color to a dye, returning `Ok(_)` for an exact match, or `Err(_)` for the closest match.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{Dye, Rgb};
    /// # use std::convert::identity;
    ///
    /// assert_eq!(Dye::AppleGreen.color(), Rgb::new(155, 179, 99));
    /// assert_eq!(Dye::try_from(Rgb::new(155, 179, 99)), Ok(Dye::AppleGreen));
    /// assert_eq!(Dye::try_from(Rgb::new(155, 179, 98)), Err(Dye::AppleGreen));
    ///
    /// // use `std::convert::identity` if you don't care about exact matches
    /// assert_eq!(Dye::try_from(Rgb::WHITE).unwrap_or_else(identity), Dye::LotusPink);
    /// assert_eq!(Dye::try_from(Rgb::BLACK).unwrap_or_else(identity), Dye::InkBlue);
    /// ```
    fn try_from(value: Rgb) -> Result<Dye, Self::Error> {
        let mut iter = Dye::VALUES.iter();

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

    /// The closest match if there is no exact match.
    type Error = Dye;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn dyes_in_self_category() {
        assert_eq!(Dye::VALUES.len(), Category::VALUES.iter().map(|category| category.dyes().len()).sum());

        for category in Category::VALUES {
            assert!(category.dyes().iter().all(|dye| dye.category() == category));
        }
    }

    #[test]
    pub fn dyes_epsilon() {
        let mut min = Rgb::new(0, 0, 0).distance(Rgb::new(255, 255, 255)) + 1;

        for dye in Dye::VALUES {
            let mut others = Dye::VALUES.iter().copied().filter(|d| *d != dye);

            let mut epsilon = {
                let other = others.next().unwrap();

                (dye.color().distance(other.color()), other)
            };

            for other in others {
                let d = other.distance(epsilon.1);

                if d < epsilon.0 {
                    epsilon = (d, other);
                }
            }

            if epsilon.0 < min {
                min = epsilon.0;
            }
        }

        assert_eq!(min, Dye::EPSILON);
    }
}
