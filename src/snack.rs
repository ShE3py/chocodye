use std::{array, fmt};
use std::fmt::Formatter;
use std::num::NonZeroU64;
use std::ops::Neg;

use crate::Rgb;

#[cfg(feature = "fluent")]
use crate::{FluentBundle, message};

/// A type of bitter fruit that changes the hue of the chocobos that eat it.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Snack {
    /// Xelphatol Apples are found growing in the Ixali homelands. Increases red hue, but reduces blue and green hues.
    Apple = 0,

    /// Mamook Pears are found growing in the arid soils of Mamook. Increases green hue, but reduces red and blue hues.
    Pear = 1,

    /// O'Ghomoro Berries are found growing on the volcanic soil of O'Ghomoro. Increases blue hue, but reduces red and green hues.
    Berries = 2,

    /// Doman Plums are found growing in the forests of far eastern Doma. Increases green and blue hues, but reduces red hue.
    Plum = 3,

    /// Valfruits are found growing on the distant Isle of Val. Increases red and blue hues, but reduces green hue.
    Fruit = 4,

    /// Cieldalaes Pineapples are found growing on the Rhotano Sea's Cieldaleas islands. Increases red and green hues, but reduces blue hue.
    Pineapple = 5
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
    #[must_use]
    #[inline]
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
    #[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
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
    #[must_use]
    #[inline]
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
    #[must_use]
    #[inline]
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
    #[inline]
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

#[cfg(test)]
mod test {
    use super::*;
    
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
}
