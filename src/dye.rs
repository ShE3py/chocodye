#[cfg(feature = "fluent")]
use crate::{FluentBundle, message};

#[cfg(all(feature = "fluent", feature = "truecolor"))]
use crate::ansi_text;

use crate::Rgb;

include!(concat!(env!("OUT_DIR"), "/dye.rs"));

impl Dye {
    /// The smallest distance between two dyes. Used to optimize search algorithms.
    pub const EPSILON: u32 = 89;

    /// The chocobos' default color.
    pub const DEFAULT_CHOCOBO_COLOR: Dye = Dye::DesertYellow;

    /// Computes the [squared Euclidian distance](https://en.wikipedia.org/wiki/Euclidean_distance#Squared_Euclidean_distance)
    /// between `self` and `other`. Does *not* take human perception into consideration. Useful for intermediate algorithms.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Dye;
    ///
    /// assert_eq!(Dye::SnowWhite.distance(Dye::SootBlack), 97278);
    /// assert_eq!(Dye::ShadowBlue.distance(Dye::CurrantPurple), 290);
    /// ```
    #[must_use]
    pub const fn distance(self, other: Dye) -> u32 {
        self.color().distance(other.color())
    }

    /// Computes the [luma](https://en.wikipedia.org/wiki/Luma_(video)), the brightness of `self`.
    /// Takes human perception into account. Useful for sorting dyes.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Dye;
    ///
    /// assert_eq!(Dye::SnowWhite.luma(), 222);
    /// assert_eq!(Dye::SootBlack.luma(), 40);
    ///
    /// assert!(Dye::HunterGreen.luma() > Dye::WineRed.luma()); // Humans are more sensitive to green.
    /// ```
    #[must_use]
    pub fn luma(self) -> u8 {
        self.color().luma()
    }

    /// Returns the localized name of `self`'s color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{{Dye, Lang}};
    ///
    /// assert_eq!(Dye::RegalPurple.color_name(&Lang::French.into_bundle()), "byzantium");
    /// ```
    #[cfg(feature = "fluent")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
    #[must_use]
    pub fn color_name(self, bundle: &FluentBundle) -> &str {
        message!(bundle, self.short_name())
    }

    /// Returns the localized name of `self`'s color with [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit) for display in `stdout`.
    ///
    /// For more documentation, check the [`ansi_text`] function. This function is also used in the `truecolor` example.
    #[cfg(all(feature = "fluent", feature = "truecolor"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "fluent", feature = "truecolor"))))]
    #[must_use]
    pub fn ansi_color_name(self, bundle: &FluentBundle) -> String {
        ansi_text(self.color(), self.color_name(bundle))
    }

    /// Parses a localized color name into its original [`Dye`].
    ///
    /// Eszetts must have been replaced by "ss". The current implementation is case-insensitive,
    /// but no diacritic-insensitve. Future implementations may be more permissive.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{{Dye, Lang}};
    ///
    /// let de = Lang::German.into_bundle();
    ///
    /// assert_eq!(Dye::from_str(&de, "Ul'dahbraun"), Some(Dye::UlBrown));    // exact match
    /// assert_eq!(Dye::from_str(&de, "Ul dahbraun"), None);                  // missing apostrophe
    /// assert_eq!(Dye::from_str(&de, "tÜrkIS"), Some(Dye::TurquoiseGreen));  // case is ignored
    /// assert_eq!(Dye::from_str(&de, "Turkis"), None);                       // missing umlaut
    /// assert_eq!(Dye::from_str(&de, "Russschwarz"), Some(Dye::SootBlack));  // `ß` was replaced by `ss`
    /// assert_eq!(Dye::from_str(&de, "Rußschwarz"), None);                   // `ß` wasn't replaced by `ss`
    /// ```
    #[cfg(feature = "fluent")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
    #[must_use]
    pub fn from_str(bundle: &FluentBundle, color_name: &str) -> Option<Dye> {
        let s = color_name.to_lowercase();

        Dye::VALUES.into_iter().find(|dye| dye.color_name(bundle).replace('ß', "ss").to_lowercase() == s)
    }
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
