use std::env;
use std::fmt::{self, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use crate::rgb::Rgb;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};

#[path = "../src/rgb.rs"]
pub mod rgb;

fn main() {
    println!("cargo:rerun-if-changed=dyes.xml");

    let mut dyes = match quick_xml::de::from_str::<Dyes>(include_str!("dyes.xml")) {
        Ok(v) => v,
        Err(e) => panic!("cannot deserialize `dyes.xml`: {e}")
    };

    // Excludes non-choco
    for cat in &mut dyes.categories {
        cat.dyes.retain(|dye| dye.choco);
    }

    if let Err(e) = codegen(&dyes) {
        panic!("cannot codegen `dyes.rs`: {e}");
    }
}

#[derive(Deserialize)]
struct Dyes {
    #[serde(rename = "category")]
    categories: Vec<Category>
}

#[derive(Deserialize)]
struct Category {
    #[serde(rename = "@name")]
    name: Name,

    #[serde(rename = "@color", deserialize_with = "deserialize_rgb")]
    color: Rgb,

    #[serde(rename = "dye", default)]
    dyes: Vec<Dye>
}

#[derive(Deserialize)]
struct Dye {
    #[serde(rename = "@name")]
    name: Name,

    #[serde(rename = "@color", deserialize_with = "deserialize_rgb")]
    color: Rgb,

    #[serde(rename = "@choco", default = "default_choco")]
    choco: bool,
}

struct Name {
    /// kebab-case
    key: String,

    /// PascalCase
    rust: String,
}

fn codegen(dyes: &Dyes) -> io::Result<()> {
    let mut path = PathBuf::from(env::var_os("OUT_DIR").expect("`OUT_DIR` is not defined"));
    path.push("dyes.rs");

    let file = File::create(path)?;
    let mut buf = BufWriter::new(file);

    codegen_enum_dye(dyes, &mut buf)?;
    codegen_enum_category(dyes, &mut buf)?;

    Ok(())
}

fn codegen_enum_dye(data: &Dyes, buf: &mut impl Write) -> io::Result<()> {
    let flat_dyes: Vec<_> = data.categories
        .iter()
        .flat_map(|category| &category.dyes)
        .collect();

    writeln!(
        buf,
        include_str!("enum.Dye.rs"),

        variants = flat_dyes.iter()
            .map(|dye| format!(
                include_str!("enum.Dye.variant.rs"),
                color = dye.color,
                variant = dye.name.rust,
            ))
            .collect::<Vec<_>>().join("\n"),

        associatedconstant_VALUES = flat_dyes.iter()
            .map(|dye| format!(
                "Dye::{variant}",
                variant = dye.name.rust,
            ))
            .collect::<Vec<_>>().join(",\n        "),

        method_category = data.categories.iter()
             .map(|category| format!(
                 "{dyes} => Category::{variant}",
                 dyes = category.dyes.iter()
                     .map(|dye| dye.name.rust.as_str())
                     .collect::<Vec<_>>().join(" | "),
                 variant = &category.name.rust,
             ))
             .collect::<Vec<_>>()
             .join(",\n            "),

        method_color = flat_dyes.iter()
            .map(|dye| format!(
                "Dye::{variant} => Rgb::new({r}, {g}, {b})",
                variant = dye.name.rust,
                r = dye.color.r,
                g = dye.color.g,
                b = dye.color.b,
            ))
            .collect::<Vec<_>>().join(",\n            "),

        method_short_names = flat_dyes.iter()
            .map(|dye| format!(
                "Dye::{variant} => {key:?}",
                variant = dye.name.rust,
                key = dye.name.key
            ))
            .collect::<Vec<_>>().join(",\n            ")
    )
}

fn codegen_enum_category(data: &Dyes, buf: &mut impl Write) -> io::Result<()> {
    writeln!(
        buf,
        include_str!("enum.Category.rs"),

        variants = data.categories.iter()
            .map(|category| category.name.rust.as_str())
            .collect::<Vec<_>>().join(",\n    "),

        associatedconstant_VALUES = data.categories.iter()
            .map(|category| format!(
                "Category::{variant}",
                variant = category.name.rust
            ))
            .collect::<Vec<_>>().join(",\n        "),

        method_dyes = data.categories
            .iter()
            .map(|category| format!(
                "Category::{variant} => &[{dyes}]",
                variant = category.name.rust,
                dyes = category.dyes.iter()
                    .map(|dye| dye.name.rust.as_str())
                    .collect::<Vec<_>>().join(", ")
            ))
            .collect::<Vec<_>>()
            .join(",\n            "),

        method_color = data.categories.iter()
            .map(|category| format!(
                "Category::{variant} => Rgb::new({r}, {g}, {b})",
                variant = category.name.rust,
                r = category.color.r,
                g = category.color.g,
                b = category.color.b,
            ))
            .collect::<Vec<_>>().join(",\n            "),

        method_short_names = data.categories.iter()
            .map(|category| format!(
                "Category::{variant} => {key:?}",
                variant = category.name.rust,
                key = category.name.key,
            ))
            .collect::<Vec<_>>().join(",\n            ")
    )?;

    Ok(())
}

fn deserialize_rgb<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Rgb, D::Error> {
    struct RgbVisitor;
    impl Visitor<'_> for RgbVisitor {
        type Value = Rgb;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
            formatter.write_str("a hex color")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            Rgb::from_hex(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(RgbVisitor)
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct NameVisitor;
        impl Visitor<'_> for NameVisitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a str")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Name {
                    key: v.to_owned(),
                    rust: v.split('-').map(|word| word[..1].to_ascii_uppercase() + &word[1..]).collect(),
                })
            }
        }

        deserializer.deserialize_str(NameVisitor)
    }
}

const fn default_choco() -> bool {
    true
}
