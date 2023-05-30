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

/// `message!(bundle, id, { $(key = value),+ })`
#[macro_export]
macro_rules! message {
    ($bundle:expr, $id:expr) => {
        $crate::__format_message($bundle, $id, None)
    };

    ($bundle:expr, $id:expr, { $($k:literal = $v:expr),+ }) => {{
        let mut args = fluent::FluentArgs::new();
        $(args.set($k, $v);)+

        $crate::__format_message($bundle, $id, Some(args))
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Lang {
    English,
    French,
    German,
    Japanese
}

impl Lang {
    pub const fn values() -> &'static [Lang] {
        use crate::Lang::*;

        &[English, French, German, Japanese]
    }

    pub const fn short_code(self) -> &'static str {
        match self {
            Lang::English  => "en",
            Lang::French   => "fr",
            Lang::German   => "de",
            Lang::Japanese => "jp"
        }
    }

    pub const fn file(self) -> &'static str {
        match self {
            Lang::English  => include_str!("ftl/en.ftl"),
            Lang::French   => include_str!("ftl/fr.ftl"),
            Lang::German   => include_str!("ftl/de.ftl"),
            Lang::Japanese => include_str!("ftl/jp.ftl")
        }
    }

    pub const fn langid(self) -> LanguageIdentifier {
        match self {
            Lang::English  => langid!("en"),
            Lang::French   => langid!("fr"),
            Lang::German   => langid!("de"),
            Lang::Japanese => langid!("jp")
        }
    }

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

pub type FluentBundle = fluent::FluentBundle<FluentResource>;

impl TryFrom<Lang> for FluentBundle {
    type Error = (FluentResource, Vec<ParserError>);

    fn try_from(value: Lang) -> Result<Self, Self::Error> {
        let mut bundle = FluentBundle::new(vec![value.langid()]);
        let res = FluentResource::try_new(value.file().to_owned())?;

        bundle.add_resource_overriding(res);
        Ok(bundle)
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_code())
    }
}

impl FromStr for Lang {
    type Err = ParseLangError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Lang::English),
            "fr" => Ok(Lang::French),
            "de" => Ok(Lang::German),
            "jp" => Ok(Lang::Japanese),
            _ => Err(ParseLangError)
        }
    }
}

impl From<Lang> for LanguageIdentifier {
    fn from(value: Lang) -> LanguageIdentifier {
        value.langid()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ParseLangError;

impl fmt::Display for ParseLangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("unknow short code")
    }
}

impl Error for ParseLangError {}
