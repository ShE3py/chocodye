use std::error::Error;
use std::fmt::{self, Formatter};
use std::num::ParseIntError;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

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

    pub const fn distance(self, other: Rgb) -> u32 {
        let dx = (self.r as i32) - (other.r as i32);
        let dy = (self.g as i32) - (other.g as i32);
        let dz = (self.b as i32) - (other.b as i32);

        (dx * dx) as u32 + (dy * dy) as u32 + (dz * dz) as u32
    }
}

impl From<u32> for Rgb {
    fn from(value: u32) -> Rgb {
        Rgb {
            r: ((value >> 24) & 0xFF) as u8,
            g: ((value >> 16) & 0xFF) as u8,
            b: ((value >> 8) & 0xFF) as u8
        }
    }
}

impl From<Rgb> for u32 {
    fn from(value: Rgb) -> u32 {
        ((value.r as u32) << 24) | ((value.g as u32) << 16) | ((value.b as u32) << 8) | 0xFF
    }
}

impl Default for Rgb {
    fn default() -> Rgb {
        Rgb::new(0, 0, 0)
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

#[derive(Debug, Clone)]
pub enum ParseHexError {
    BadLen,
    MissingHash,
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
