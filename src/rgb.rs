use std::error::Error;
use std::fmt::{self, Formatter};
use std::num::ParseIntError;

/// A color represented by three `u8` components.
///
/// This struct implements [`Debug`](fmt::Debug), [`LowerHex`](fmt::LowerHex) and
/// [`UpperHex`](fmt::UpperHex), but no [`Display`](fmt::Display).
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rgb {
    /// The red component.
    pub r: u8,

    /// The green component.
    pub g: u8,

    /// The blue component.
    pub b: u8
}

impl Rgb {
    /// `(255, 255, 255)`
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    /// `(0, 0, 0)`
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);

    /// `(255, 0, 0)`
    pub const RED: Rgb = Rgb::new(255, 0, 0);

    /// `(0, 255, 0)`
    pub const GREEN: Rgb = Rgb::new(0, 255, 0);

    /// `(0, 0, 255)`
    pub const BLUE: Rgb = Rgb::new(0, 0, 255);

    /// `(255, 255, 0)`
    pub const YELLOW: Rgb = Rgb::new(255, 255, 0);

    /// `(0, 255, 255)`
    pub const CYAN: Rgb = Rgb::new(0, 255, 255);

    /// `(255, 0, 255)`
    pub const MAGENTA: Rgb = Rgb::new(255, 0, 255);

    /// Creates a new color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::new(5, 7, 11), Rgb { r: 5, g: 7, b: 11 });
    /// ```
    #[must_use]
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    /// Creates a new color with all three components set to the same value.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::gray(127), Rgb::new(127, 127, 127));
    /// ```
    #[must_use]
    #[inline]
    pub const fn gray(rgb: u8) -> Rgb {
        Rgb { r: rgb, g: rgb, b: rgb }
    }

    /// Parses a hex color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::from_hex("#ffffff"), Ok(Rgb::new(255, 255, 255)));
    /// assert!(Rgb::from_hex("#fff").is_err());
    /// assert!(Rgb::from_hex("ffffff").is_err());
    /// ```
    #[expect(clippy::missing_errors_doc, clippy::indexing_slicing)]
    pub fn from_hex(s: &str) -> Result<Rgb, ParseHexError> {
        if s.len() != 7 {
            Err(ParseHexError::BadLen)
        }
        else if s.as_bytes()[0] != b'#' {
            Err(ParseHexError::MissingHash)
        }
        else {
            Ok((u32::from_str_radix(&s[1..7], 16)? << 8).into())
        }
    }

    /// Checked addition with three signed components.
    /// Computes `self + rgb`, returning `None` if overflow occured.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::new(20, 30, 40).checked_add_signed(2, -2, 0), Some(Rgb::new(22, 28, 40)));
    /// assert_eq!(Rgb::new(10, 2, 250).checked_add_signed(4, -5, 1), None); // `g` would underflow.
    /// ```
    #[must_use]
    #[inline]
    pub const fn checked_add_signed(self, r: i8, g: i8, b: i8) -> Option<Rgb> {
        // FIXME: use const ? when stable
        macro_rules! checked_add_signed {
            ($lhs:expr, $rhs:expr) => {
                match $lhs.checked_add_signed($rhs) {
                    Some(v) => v,
                    None => return None
                }
            };
        }

        Some(Rgb {
            r: checked_add_signed!(self.r, r),
            g: checked_add_signed!(self.g, g),
            b: checked_add_signed!(self.b, b),
        })
    }

    /// Computes the [squared Euclidian distance](https://en.wikipedia.org/wiki/Euclidean_distance#Squared_Euclidean_distance)
    /// between `self` and `other`. Does *not* take human perception into consideration. Useful for intermediate algorithms.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::WHITE.distance(Rgb::WHITE), 0);
    /// assert_eq!(Rgb::gray(8).distance(Rgb::gray(16)), Rgb::gray(24).distance(Rgb::gray(16)));
    /// ```
    #[must_use]
    #[inline]
    pub const fn distance(self, other: Rgb) -> u32 {
        let dx = (self.r as i32) - (other.r as i32);
        let dy = (self.g as i32) - (other.g as i32);
        let dz = (self.b as i32) - (other.b as i32);

        (dx * dx) as u32 + (dy * dy) as u32 + (dz * dz) as u32
    }

    /// Computes the [luma](https://en.wikipedia.org/wiki/Luma_(video)), the brightness of `self`.
    /// Takes human perception into account. Useful for sorting colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::WHITE.luma(), 255);
    /// assert_eq!(Rgb::BLACK.luma(), 0);
    ///
    /// assert_eq!(Rgb::new(10, 20, 30).luma(), 18);
    /// assert_eq!(Rgb::new(20, 40, 60).luma(), 36);
    ///
    /// assert!(Rgb::GREEN.luma() > Rgb::RED.luma()); // Humans are more sensitive to green.
    /// ```
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_truncation, clippy::suboptimal_flops)]
    pub fn luma(self) -> u8 {
        (0.299 * (self.r as f32) + 0.587 * (self.g as f32) + 0.114 * (self.b as f32)) as u8
    }

    /// Converts this color into a gray shade. Takes human perception into account.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::RED.grayscale(), Rgb::gray(76));
    /// assert_eq!(Rgb::WHITE.grayscale(), Rgb::WHITE);
    /// assert_eq!(Rgb::BLACK.grayscale(), Rgb::BLACK);
    /// ```
    #[must_use]
    #[inline]
    pub fn grayscale(self) -> Rgb {
        Rgb::gray(self.luma())
    }
}

impl From<u32> for Rgb {
    /// Converts an `u32` in `RRGGBBAA` format to its corresponding color. The alpha bits are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(Rgb::from(0x01020300), Rgb::new(1, 2, 3));
    /// ```
    #[inline]
    fn from(value: u32) -> Rgb {
        Rgb {
            r: ((value >> 24) & 0xFF) as u8,
            g: ((value >> 16) & 0xFF) as u8,
            b: ((value >> 8) & 0xFF) as u8
        }
    }
}

impl From<Rgb> for u32 {
    /// Converts this color to an `u32` in `RRGGBBAA` format. The alpha bits are set to `0xFF`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(u32::from(Rgb::new(1, 2, 3)), 0x010203FF);
    /// assert_ne!(u32::from(Rgb::from(0x0ABCDEF0)), 0x0ABCDEF0); // The alpha bits are lost.
    /// ```
    #[inline]
    fn from(value: Rgb) -> u32 {
        ((value.r as u32) << 24) | ((value.g as u32) << 16) | ((value.b as u32) << 8) | 0xFF
    }
}

impl Default for Rgb {
    /// The default color is arbitrarily set to `#5bcefa`, a light blue.
    #[inline]
    fn default() -> Rgb {
        Rgb::new(91, 206, 250)
    }
}

impl fmt::Debug for Rgb {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Rgb")
            .field(&self.r)
            .field(&self.g)
            .field(&self.b)
            .finish()
    }
}

impl fmt::LowerHex for Rgb {
    /// Formats `self` as a hex color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(format!("{:x}", Rgb::RED), "#ff0000");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl fmt::UpperHex for Rgb {
    /// Formats `self` as a hex color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Rgb;
    ///
    /// assert_eq!(format!("{:X}", Rgb::RED), "#FF0000");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// An error that can be returned when parsing a hexadecimal color.
///
/// This error is used as the error type for the [`Rgb::from_hex`] function.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ParseHexError {
    /// The string length is not seven (`#rrggbb`).
    BadLen,

    /// The string does not begin with a hashtag (`#`).
    MissingHash,

    /// The string contains an invalid digit.
    BadInt(ParseIntError)
}

impl fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseHexError::BadLen => write!(f, "bad length"),
            ParseHexError::MissingHash => write!(f, "missing `#` prefix"),
            ParseHexError::BadInt(e) => fmt::Display::fmt(e, f)
        }
    }
}

impl From<ParseIntError> for ParseHexError {
    fn from(e: ParseIntError) -> ParseHexError {
        ParseHexError::BadInt(e)
    }
}

impl Error for ParseHexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseHexError::BadInt(e) => Some(e),
            _ => None
        }
    }
}
