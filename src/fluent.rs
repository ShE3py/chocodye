#![cfg(feature = "fluent")]

use std::borrow::{Borrow, Cow};
use std::error::Error;
use std::fmt::{self, Formatter};
use std::str::FromStr;

use fluent::{FluentArgs, FluentResource};
use fluent::memoizer::MemoizerKind;
use fluent::resolver::Scope;
use fluent_syntax::parser::ParserError;
use log::error;
use unic_langid::{langid, LanguageIdentifier};

/// Formats a Fluent message fail-safely. Missing keys are formatted arbitrarily.
///
/// Messages without arguments are evaluated to [`&str`], while messages with arguments
/// are evaluated to [`String`].
///
/// # Examples
///
/// ```
/// use chocodye::{Lang, message};
///
/// let bundle = Lang::English.into_bundle();
///
/// assert_eq!(message!(&bundle, "sky-blue"), "Sky Blue");
/// assert_eq!(message!(&bundle, "pear", { "quantity" = 1 }), "\u{2068}1\u{2069} Mamook Pear");
///
/// assert_eq!(message!(&bundle, "missing-key", { "foo" = "bar" }), "missing-key(bar)");
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
macro_rules! message {
    ($bundle:expr, $id:expr $(, {})?) => {
        {
            let key: &'static str = $id;

            match $crate::__format_message($bundle, key, None) {
                ::std::borrow::Cow::Borrowed(s) => s,
                ::std::borrow::Cow::Owned(_string) => {
                    #[cfg(debug_assertions)]
                    { ::std::unreachable!("`message!(_, {:?})` should be `Cow::Borrowed(_)`, got `Cow::Owned({:?})`", key, _string) }

                    #[cfg(not(debug_assertions))]
                    { key }
                }
            }
        }
    };

    ($bundle:expr, $id:expr, { $($k:literal = $v:expr),+ }) => {{
        let mut args = fluent::FluentArgs::new();
        $(args.set($k, $v);)+

        $crate::__format_message($bundle, $id, Some(args)).into_owned()
    }};
}

#[doc(hidden)]
pub fn __format_message<'a, R, M>(bundle: &'a fluent::bundle::FluentBundle<R, M>, id: &'static str, args: Option<FluentArgs<'a>>) -> Cow<'a, str> where R: Borrow<FluentResource>, M: MemoizerKind {
    match bundle.get_message(id) {
        Some(msg) => match msg.value() {
            Some(pattern) => {
                let mut errors = Vec::new();

                match &args {
                    Some(args) => {
                        let result = bundle.format_pattern(pattern, Some(args), &mut errors);

                        if errors.is_empty() {
                            return Cow::Owned(result.into_owned());
                        }
                    },
                    None => {
                        let result = bundle.format_pattern(pattern, None, &mut errors);

                        if errors.is_empty() {
                            return result;
                        }
                    }
                }

                error!(target: "fluent", "unable to format message `{}`", id);

                for error in errors {
                    error!(target: "fluent", "{}", error);
                }
            },
            None => error!(target: "fluent", "message `{}` has no value", id)
        },
        None => error!(target: "fluent", "missing message `{}`", id)
    }

    if let Some(args) = args {
        let scope = Scope::new(bundle, None, None);
        let args = args.into_iter().map(|(_, v)| v.as_string(&scope)).collect::<Vec<_>>().join(", ");

        Cow::Owned(format!("{}({})", id, args))
    }
    else {
        Cow::Borrowed(id)
    }
}

/// A language officially supported by *Final Fantasy XIV*.
/// Can be converted into a [`FluentBundle`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
pub enum Lang {
    English,
    French,
    German,
    Japanese
}

impl Lang {
    /// Contains all four `Lang` variants.
    pub const VALUES: [Lang; 4] = [
        Lang::English,
        Lang::French,
        Lang::German,
        Lang::Japanese
    ];

    /// Returns the two-letter [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
    /// of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Lang;
    ///
    /// assert_eq!(Lang::German.short_code(), "de");
    /// ```
    pub const fn short_code(self) -> &'static str {
        match self {
            Lang::English  => "en",
            Lang::French   => "fr",
            Lang::German   => "de",
            Lang::Japanese => "jp"
        }
    }

    /// Returns the [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
    /// of `self`.
    pub const fn langid(self) -> LanguageIdentifier {
        match self {
            Lang::English  => langid!("en"),
            Lang::French   => langid!("fr"),
            Lang::German   => langid!("de"),
            Lang::Japanese => langid!("jp")
        }
    }

    /// Returns the Fluent translation resource of `self`.
    pub const fn file(self) -> &'static str {
        match self {
            Lang::English  => include_str!("ftl/en.ftl"),
            Lang::French   => include_str!("ftl/fr.ftl"),
            Lang::German   => include_str!("ftl/de.ftl"),
            Lang::Japanese => include_str!("ftl/jp.ftl")
        }
    }

    /// Parses the translation resource of `self` into a new [`FluentBundle`].
    /// Returns an empty bundle on error, but this shouldn't happen since the file is located in the read-only data segment.
    pub fn into_bundle(self) -> FluentBundle {
        match self.try_into() {
            Ok(bundle) => bundle,
            Err((_, errors)) => {
                error!(target: "lang", "unable to load bundle `{}`:", self.short_code());

                for error in errors {
                    error!(target: "lang", "{}", error);
                }

                FluentBundle::new(vec![self.langid()])
            }
        }
    }
}

/// A collection of messages for a given language. Obtained from [`Lang::into_bundle`].
#[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
pub type FluentBundle = fluent::FluentBundle<FluentResource>;

impl TryFrom<Lang> for FluentBundle {
    // the `Error` type is further down so that it stays below the function in the generated documentation,
    // so that the reader reads the type description after the fn description.

    /// Parses the translation resource of `value` into a new [`FluentBundle`].
    /// Returns both the resource and a vec of errors in case of error.
    fn try_from(value: Lang) -> Result<Self, Self::Error> {
        let mut bundle = FluentBundle::new(vec![value.langid()]);
        let res = FluentResource::try_new(value.file().to_owned())?;

        bundle.add_resource_overriding(res);
        Ok(bundle)
    }

    /// The tuple returned in the event of a parse error.
    type Error = (FluentResource, Vec<ParserError>);
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_code())
    }
}

impl FromStr for Lang {
    // the `Error` type is further down so that it stays below the function in the generated documentation,
    // so that the reader reads the type description after the fn description.

    /// Parses the two-letter [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
    /// of a `Lang`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{Lang, ParseLangError};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Lang::from_str("jp"), Ok(Lang::Japanese));
    /// assert_eq!(Lang::from_str("ja"), Err(ParseLangError));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Lang::English),
            "fr" => Ok(Lang::French),
            "de" => Ok(Lang::German),
            "jp" => Ok(Lang::Japanese),
            _ => Err(ParseLangError)
        }
    }

    /// The type returned if there's no [`Lang`] associated with a given `&str`.
    type Err = ParseLangError;
}

impl From<Lang> for LanguageIdentifier {
    /// Returns [`Lang::langid`].
    fn from(value: Lang) -> LanguageIdentifier {
        value.langid()
    }
}

/// An error that can be returned when parsing an [Unicode Language Identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier).
///
/// This error is used as the error type for the [`Lang::from_str`] function.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
pub struct ParseLangError;

impl fmt::Display for ParseLangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("unknow short code")
    }
}

impl Error for ParseLangError {}
