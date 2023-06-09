
fn main() {
    println!("cargo:rerun-if-changed=src/xml/dyes.xml");

    dyes::codegen();
}

mod dyes {
    use std::{io, iter};
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::PathBuf;

    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Dyes {
        #[serde(rename = "category")]
        categories: Vec<Category>
    }

    #[derive(Deserialize)]
    struct Category {
        #[serde(rename = "@name")]
        name: String,

        #[serde(rename = "@stain")]
        stain: String,

        #[serde(rename = "dye", default)]
        dyes: Vec<Dye>
    }

    #[derive(Deserialize)]
    struct Dye {
        #[serde(rename = "@name")]
        name: String,

        #[serde(rename = "@stain")]
        stain: String,

        #[serde(rename = "@choco", default = "default_choco")]
        choco: bool,
    }

    const fn default_choco() -> bool {
        true
    }

    impl Dyes {
        pub fn codegen(&self) -> io::Result<()> {
            let mut path = PathBuf::from(std::env::var_os("OUT_DIR").expect("`OUT_DIR` is not defined"));
            path.push("dye.rs");

            let file = File::create(path)?;
            let mut buf = BufWriter::new(file);

            self.codegen_dyes(&mut buf)?;
            self.codegen_category(&mut buf)?;

            Ok(())
        }

        fn codegen_dyes(&self, buf: &mut impl Write) -> io::Result<()> {
            let dyes: Vec<_> = self.categories
                .iter()
                .flat_map(|category| &category.dyes)
                .filter(|dye| dye.choco)
                .collect();

            let variants: Vec<_> = dyes
                .iter()
                .map(|dye| make_pascal_case(&dye.name))
                .collect();

            writeln!(buf,
r#"
/// A color that can be found as the plumage of a chocobo.
///
/// Some dyes, such as vanilla yellow, are not included in this enum.
///
/// As the build script has no access to [`Rgb`], documentation of variants is rather feeble.
/// Please open an issue on GitHub if you wish to use this enum in another crate not related
/// to chocobo dyeing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Dye {{
    {variants}
}}

impl Dye {{
    /// The smallest distance between two dyes. Used to optimize search algorithms.
    pub const EPSILON: u32 = 269;

    /// The chocobos' default color.
    pub const DEFAULT_CHOCOBO_COLOR: Dye = Dye::DesertYellow;

    /// Contains all eighty-five `Dye` variants.
    pub const VALUES: [Dye; 85] = [
        {values}
    ];

    /// Returns the dye category of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{{Category, Dye}};
    ///
    /// assert_eq!(Dye::CeruleumBlue.category(), Category::Blue);
    /// ```
    pub const fn category(self) -> Category {{
        match self {{
            {categories}
        }}
    }}

    /// Returns the color of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{{Dye, Rgb}};
    ///
    /// assert_eq!(Dye::DesertYellow.color(), Rgb::new(219, 180, 87));
    /// ```
    pub const fn color(self) -> Rgb {{
        match self {{
            {rgbs}
        }}
    }}

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
    pub const fn distance(self, other: Dye) -> u32 {{
        self.color().distance(other.color())
    }}

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
    pub fn luma(self) -> u8 {{
        self.color().luma()
    }}

    /// Returns the variant name of `self` in kebab-case.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Dye;
    ///
    /// assert_eq!(Dye::OpoOpoBrown.short_name(), "opo-opo-brown");
    /// ```
    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}

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
    pub fn color_name(self, bundle: &FluentBundle) -> &str {{
        message!(bundle, self.short_name())
    }}

    /// Returns the localized name of `self`'s color with [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit) for display in `stdout`.
    ///
    /// For more documentation, check the [`ansi_text`] function. This function is also used in the `truecolor` example.
    #[cfg(all(feature = "fluent", feature = "truecolor"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "fluent", feature = "truecolor"))))]
    pub fn ansi_color_name(self, bundle: &FluentBundle) -> String {{
        ansi_text(self.color(), self.color_name(bundle))
    }}

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
    pub fn from_str(bundle: &FluentBundle, color_name: &str) -> Option<Dye> {{
        from_str_impl(bundle, color_name)
    }}
}}"#,
                     variants = dyes.iter().zip(&variants).map(|(dye, variant)| format!("/// `{}`\n\t{variant}", dye.stain)).collect::<Vec<_>>().join(",\n\n\t"),
                     values = variants.iter().map(|dye| format!("Dye::{dye}")).collect::<Vec<_>>().join(",\n\t\t"),

                     categories = self.categories
                         .iter()
                         .flat_map(|category| iter::repeat(category).zip(&category.dyes))
                         .filter(|(_, dye)| dye.choco)
                         .map(|(category, dye)| format!("Dye::{} => Category::{}", make_pascal_case(&dye.name), make_pascal_case(&category.name)))
                         .collect::<Vec<_>>()
                         .join(",\n\t\t\t"),

                     rgbs = dyes.iter().zip(variants.iter()).map(|(dye, name)| format!("Dye::{name} => {}", make_rgb(&dye.stain))).collect::<Vec<_>>().join(",\n\t\t\t"),
                     names = self.categories.iter().flat_map(|category| &category.dyes).filter(|dye| dye.choco).map(|dye| format!("Dye::{} => {:?}", make_pascal_case(&dye.name), &dye.name)).collect::<Vec<_>>().join(",\n\t\t\t")
            )
        }

        fn codegen_category(&self, buf: &mut impl Write) -> io::Result<()> {
            let categories: Vec<_> = self.categories
                .iter()
                .map(|category| make_pascal_case(&category.name)).
                collect();

            writeln!(buf, r#"
/// A category of dyes with similar hues.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Category {{
    {variants}
}}

impl Category {{
    /// Contains all seven `Category` variants.
    pub const VALUES: [Category; 7] = [
        {values}
    ];

    /// Returns all the dyes belonging to `self`. Dyes belong to one and only one category.
    pub const fn dyes(self) -> &'static [Dye] {{
        use Dye::*;

        match self {{
            {dyes}
        }}
    }}

    /// Returns a color representing `self`. Does not necessarily correspond to a dye.
    pub const fn color(self) -> Rgb {{
        match self {{
            {rgbs}
        }}
    }}

    /// Returns the variant name of `self` in kebab-case.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::Category;
    ///
    /// assert_eq!(Category::Purple.short_name(), "purple");
    /// ```
    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}

    /// Returns the localized name of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chocodye::{{Category, Lang}};
    ///
    /// assert_eq!(Category::Purple.full_name(&Lang::French.into_bundle()), "Teintures violettes");
    /// ```
    #[cfg(feature = "fluent")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fluent")))]
    pub fn full_name(self, bundle: &FluentBundle) -> &str {{
        message!(bundle, self.short_name())
    }}

    /// Returns the localized name of `self` with [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit) for display in `stdout`.
    ///
    /// For more documentation, check the [`ansi_text`] function. This function is also used in the `truecolor` example.
    #[cfg(all(feature = "fluent", feature = "truecolor"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "fluent", feature = "truecolor"))))]
    pub fn ansi_full_name(self, bundle: &FluentBundle) -> String {{
        ansi_text(self.color(), self.full_name(bundle))
    }}
}}"#,
                     variants = categories.join(",\n\t"),
                     values = categories.iter().map(|category| format!("Category::{category}")).collect::<Vec<_>>().join(",\n\t\t"),

                     dyes = self.categories
                    .iter()
                    .map(|category| format!(
                        "Category::{} => &[{}]",
                        make_pascal_case(&category.name),
                        category.dyes
                            .iter()
                            .filter(|dye| dye.choco)
                            .map(|dye| make_pascal_case(&dye.name))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .collect::<Vec<_>>()
                    .join(",\n\t\t\t"),

                     rgbs = self.categories.iter().map(|category| format!("Category::{} => {}", make_pascal_case(&category.name), make_rgb(&category.stain))).collect::<Vec<_>>().join(",\n\t\t\t"),
                     names = self.categories.iter().map(|category| format!("Category::{} => {:?}", make_pascal_case(&category.name), &category.name)).collect::<Vec<_>>().join(",\n\t\t\t")
            )?;

            Ok(())
        }
    }

    pub fn codegen() {
        let dyes = match quick_xml::de::from_str::<Dyes>(include_str!("src/xml/dyes.xml")) {
            Ok(v) => v,
            Err(e) => panic!("cannot deserialize `dyes.xml`: {e}")
        };

        if let Err(e) = dyes.codegen() {
            panic!("cannot codegen `dyes.rs`: {e}");
        }
    }

    fn make_pascal_case(kebab_case: &str) -> String {
        let mut pc = Vec::with_capacity(kebab_case.len());

        let mut make_upper = true;
        for mut b in kebab_case.bytes() {
            if b == b'-' {
                make_upper = true;
            }
            else {
                if make_upper {
                    b.make_ascii_uppercase();

                    make_upper = false;
                }

                pc.push(b);
            }
        }

        String::from_utf8(pc).expect("infallible conversion failed")
    }

    fn make_rgb(s: &str) -> String {
        // copied from Rgb::from_hex

        if s.len() != 7 || s.as_bytes()[0] != b'#' {
            panic!("malformed color: {:?}", s);
        }

        match u32::from_str_radix(&s[1..7], 16) {
            Ok(v) => format!("Rgb::new({}, {}, {})", (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF),
            Err(e) => panic!("malformed color: {:?}: {}", s, e)
        }
    }
}

pub enum Dye {
    SnowWhite,
}
