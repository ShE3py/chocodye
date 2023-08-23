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
        let mut dyes = Dye::VALUES;
        dyes.sort_unstable_by_key(|d| d.color().distance(value));
        
        let [best, ..] = dyes;
        
        if best.color() == value {
            Ok(best)
        }
        else {
            Err(best)
        }
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
        let mut min = Rgb::BLACK.distance(Rgb::WHITE) + 1;
        
        for a in Dye::VALUES {
            for b in Dye::VALUES {
                let d = a.distance(b);
                
                if a != b && d < min {
                    min = d;
                }
            }
        }

        assert_eq!(min, Dye::EPSILON);
    }
}
