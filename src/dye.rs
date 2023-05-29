#[cfg(feature = "fluent")]
use {fluent::{bundle::FluentBundle, FluentResource, memoizer::MemoizerKind, FluentArgs}, log::error, std::borrow::{Borrow, Cow}};

use crate::Rgb;

include!(concat!(env!("OUT_DIR"), "/dye.rs"));

impl From<Dye> for Rgb {
    fn from(dye: Dye) -> Rgb {
        dye.color()
    }
}

#[cfg(feature = "fluent")]
fn full_name<R: Borrow<FluentResource>, M: MemoizerKind>(dye: Dye, bundle: &FluentBundle<R, M>) -> String {
    let color_name = color_name(dye, &bundle);

    match bundle.get_message("dye") {
        Some(msg) => {
            match msg.value() {
                Some(pattern) => {
                    let mut args = FluentArgs::new();
                    args.set("name", color_name.clone());

                    let mut errors = Vec::new();
                    let result = bundle.format_pattern(pattern, Some(&args), &mut errors);

                    if errors.is_empty() {
                        return result.into_owned();
                    }
                    else {
                        error!(target: "fluent", "unable to format message `dye`:");

                        for error in errors {
                            error!(target: "fluent", "{}", error);
                        }
                    }
                },
                None => error!(target: "fluent", "message `dye` has no value")
            }
        },
        None => error!(target: "fluent", "missing message `dye`")
    }

    format!("{}.dye", color_name)
}

#[cfg(feature = "fluent")]
fn color_name<R: Borrow<FluentResource>, M: MemoizerKind>(dye: Dye, bundle: &FluentBundle<R, M>) -> Cow<str> {
    match bundle.get_message(dye.short_name()) {
        Some(msg) => match msg.value() {
            Some(pattern) => {
                let mut errors = Vec::new();
                let result = bundle.format_pattern(pattern, None, &mut errors);

                if errors.is_empty() {
                    return result;
                } else {
                    error!(target: "fluent", "unable to format message `{}`", dye.short_name());

                    for error in errors {
                        error!(target: "fluent", "{}", error);
                    }
                }
            },
            None => error!(target: "fluent", "message `{}` has no value", dye.short_name())
        },
        None => error!(target: "fluent", "missing message `{}`", dye.short_name())
    }

    Cow::Borrowed(dye.short_name())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn dyes_in_self_category() {
        assert_eq!(Dye::values().len(), Category::values().iter().map(|category| category.dyes().len()).sum());

        for category in Category::values() {
            assert!(category.dyes().iter().all(|dye| dye.category() == *category));
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
