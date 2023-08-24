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
    /// The closest match if there is no exact match.
    type Error = Dye;

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
        let dye = Dye::VALUES.iter().copied().min_by_key(|d| d.color().distance(value)).unwrap();
        
        if dye.color() == value {
            Ok(dye)
        }
        else {
            Err(dye)
        }
    }
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
    pub fn dye_epsilon() {
        let mut epsilon = u32::MAX;
        
        for a in Dye::VALUES {
            for b in Dye::VALUES {
                if a != b {
                    let d = a.distance(b);
                    
                    if d < epsilon {
                        epsilon = d;
                    }
                }
            }
        }

        assert_eq!(epsilon, Dye::EPSILON);
    }
}
