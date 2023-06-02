use std::ops::Neg;

use crate::{FluentBundle, message, Rgb};

/// A type of bitter fruit that changes the hue of the chocobos that eat it.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Snack {
    /// Xelphatol Apples are found growing in the Ixali homelands. Increases red hue, but reduces blue and green hues.
    Apple,

    /// Mamook Pears are found growing in the arid soils of Mamook. Increases green hue, but reduces red and blue hues.
    Pear,

    /// O'Ghomoro Berries are found growing on the volcanic soil of O'Ghomoro. Increases blue hue, but reduces red and green hues.
    Berries,

    /// Doman Plums are found growing in the forests of far eastern Doma. Increases green and blue hues, but reduces red hue.
    Plum,

    /// Valfruits are found growing on the distant Isle of Val. Increases red and blue hues, but reduces green hue.
    Fruit,

    /// Cieldalaes Pineapples are found growing on the Rhotano Sea's Cieldaleas islands. Increases red and green hues, but reduces blue hue.
    Pineapple
}

impl Snack {
    /// Contains all six `Snack` variants.
    pub const VALUES: [Snack; 6] = [
        Snack::Apple,
        Snack::Pear,
        Snack::Berries,
        Snack::Plum,
        Snack::Fruit,
        Snack::Pineapple
    ];

    /// Returns the variant name of `self` in kebab-case.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Snack;
    ///
    /// assert_eq!(Snack::Apple.short_name(), "apple");
    /// ```
    pub const fn short_name(self) -> &'static str {
        match self {
            Snack::Apple     => "apple",
            Snack::Pear      => "pear",
            Snack::Berries   => "berries",
            Snack::Plum      => "plum",
            Snack::Fruit     => "fruit",
            Snack::Pineapple => "pineapple"
        }
    }

    /// Returns the localized quantified name of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{Lang, Snack};
    ///
    /// assert_eq!(Snack::Fruit.quantified_name(&Lang::English.into_bundle(), 12), "\u{2068}12\u{2069} Valfruits");
    /// ```
    #[cfg(feature = "fluent")]
    #[cfg_attr(docrs, doc(cfg(feature = "fluent")))]
    pub fn quantified_name(self, bundle: &FluentBundle, quantity: u32) -> String {
        message!(bundle, self.short_name(), { "quantity" = quantity })
    }

    /// Returns the effect `self` will have on a chocobo's plumage.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Snack;
    ///
    /// assert_eq!(Snack::Plum.effect(), (-5, 5, 5));
    /// ```
    pub const fn effect(self) -> (i8, i8, i8) {
        match self {
            Snack::Apple     => ( 5, -5, -5),
            Snack::Pear      => (-5,  5, -5),
            Snack::Berries   => (-5, -5,  5),
            Snack::Plum      => (-5,  5,  5),
            Snack::Fruit     => ( 5, -5,  5),
            Snack::Pineapple => ( 5,  5, -5)
        }
    }

    /// Returns the color a chocobo would have if it ate this snack, or `None` if any color component had overflowed.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{Rgb, Snack};
    ///
    /// assert_eq!(Snack::Plum.alter(Rgb::new(40, 20, 70)), Some(Rgb::new(35, 25, 75)));
    /// assert_eq!(Snack::Plum.alter(Rgb::new(40, 20, 255)), None);
    /// ```
    pub const fn alter(self, color: Rgb) -> Option<Rgb> {
        let (r, g, b) = self.effect();

        color.checked_add_signed(r, g, b)
    }
}

impl Neg for Snack {
    type Output = Snack;

    /// Returns the snack that nullifies the effect of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Snack;
    ///
    /// assert_eq!(Snack::Pear.effect(), (-5, 5, -5));
    /// assert_eq!((-Snack::Pear).effect(), (5, -5, 5));
    /// ```
    fn neg(self) -> Self::Output {
        match self {
            Snack::Apple => Snack::Plum,
            Snack::Pear => Snack::Fruit,
            Snack::Berries => Snack::Pineapple,
            Snack::Plum => Snack::Apple,
            Snack::Fruit => Snack::Pear,
            Snack::Pineapple => Snack::Berries
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn neg() {
        for snack in Snack::VALUES {
            let a = snack.effect();
            let b = snack.neg().effect();

            assert_eq!(a.0 + b.0, 0);
            assert_eq!(a.1 + b.1, 0);
            assert_eq!(a.2 + b.2, 0);
        }
    }
}
