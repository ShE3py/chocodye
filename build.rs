
fn main() {
    println!("cargo:rerun-if-changed=src/xml/dyes.xml");

    dyes::codegen();
}

mod dyes {
    use std::fs::File;
    use std::io::{self, BufWriter, Write};
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
        fn codegen(&self) -> io::Result<()> {
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
#[repr(u8)]
pub enum Dye {{
    {variants}
}}

impl Dye {{
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
    #[must_use]
    #[inline]
    pub const fn category(self) -> Category {{
        use Dye::*;

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
    #[must_use]
    #[inline]
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
    /// use chocodye::Dye;
    ///
    /// assert_eq!(Dye::OpoOpoBrown.short_name(), "opo-opo-brown");
    /// ```
    #[must_use]
    #[inline]
    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}
}}"#,
                     variants = dyes.iter().zip(&variants).enumerate().map(|(i, (dye, variant))| format!("/// `{}`\n\t{variant} = {i}", dye.stain)).collect::<Vec<_>>().join(",\n\n\t"),
                     values = variants.iter().map(|dye| format!("Dye::{dye}")).collect::<Vec<_>>().join(",\n\t\t"),

                     categories = self.categories
                         .iter()
                         .map(|category| format!("{} => Category::{}", category.dyes.iter().filter(|dye| dye.choco).map(|dye| make_pascal_case(&dye.name)).collect::<Vec<_>>().join(" | "), make_pascal_case(&category.name)))
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
#[repr(u8)]
pub enum Category {{
    {variants}
}}

impl Category {{
    /// Contains all seven `Category` variants.
    pub const VALUES: [Category; 7] = [
        {values}
    ];

    /// Returns all the dyes belonging to `self`. Dyes belong to one and only one category.
    #[must_use]
    #[inline]
    pub const fn dyes(self) -> &'static [Dye] {{
        use Dye::*;

        match self {{
            {dyes}
        }}
    }}

    /// Returns a color representing `self`. Does not necessarily correspond to a dye.
    #[must_use]
    #[inline]
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
    #[must_use]
    #[inline]
    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}
}}"#,
                     variants = categories.iter().enumerate().map(|(i, category)| format!("{category} = {i}")).collect::<Vec<_>>().join(",\n\t"),
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

    pub(crate) fn codegen() {
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

        assert!(s.len() == 7 && s.as_bytes()[0] == b'#', "malformed color: {s:?}");

        match u32::from_str_radix(&s[1..7], 16) {
            Ok(v) => format!("Rgb::new({}, {}, {})", (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF),
            Err(e) => panic!("malformed color: {s:?}: {e}")
        }
    }
}
