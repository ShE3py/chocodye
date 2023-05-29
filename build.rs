
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
r#"#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Dye {{
    {variants}
}}

impl Dye {{
    pub const EPSILON: u32 = 269;

    pub const fn values() -> &'static [Dye] {{
        use Dye::*;

        &[{values}]
    }}

    pub const fn category(self) -> Category {{
        match self {{
            {categories}
        }}
    }}

    pub const fn color(self) -> Rgb {{
        match self {{
            {rgbs}
        }}
    }}

    pub const fn distance(self, other: Dye) -> u32 {{
        self.color().distance(other.color())
    }}

    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}

    #[cfg(feature = "fluent")]
    pub fn full_name<R: Borrow<FluentResource>, M: MemoizerKind>(self, bundle: &FluentBundle<R, M>) -> String {{
        full_name(self, bundle)
    }}

    #[cfg(feature = "fluent")]
    pub fn ansi_full_name<R: Borrow<FluentResource>, M: MemoizerKind>(self, bundle: &FluentBundle<R, M>) -> String {{
        ansi_text(self, &full_name(self, bundle))
    }}

    #[cfg(feature = "fluent")]
    pub fn color_name<R: Borrow<FluentResource>, M: MemoizerKind>(self, bundle: &FluentBundle<R, M>) -> Cow<str> {{
        color_name(self, bundle)
    }}

    #[cfg(feature = "fluent")]
    pub fn ansi_color_name<R: Borrow<FluentResource>, M: MemoizerKind>(self, bundle: &FluentBundle<R, M>) -> String {{
        ansi_text(self, color_name(self, bundle).as_ref())
    }}
}}"#,
                     variants = variants.join(",\n\t"),
                     values = variants.join(", "),

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Category {{
    {variants}
}}

impl Category {{
    pub const fn values() -> &'static [Category] {{
        use Category::*;

        &[{values}]
    }}

    pub const fn dyes(self) -> &'static [Dye] {{
        use Dye::*;

        match self {{
            {dyes}
        }}
    }}

    pub const fn color(self) -> Rgb {{
        match self {{
            {rgbs}
        }}
    }}

    pub const fn short_name(self) -> &'static str {{
        match self {{
            {names}
        }}
    }}
}}"#,
                     variants = categories.join(",\n\t"),
                     values = categories.join(", "),

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
