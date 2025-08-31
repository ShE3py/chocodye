/// A category of dyes with similar hues.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Category {{
    {variants}
}}

impl Category {{
    /// Contains all seven `Category` variants.
    pub const VALUES: [Category; 7] = [
        {associatedconstant_VALUES}
    ];

    /// Returns all the dyes belonging to `self`. Dyes belong to one and only one category.
    #[must_use]
    #[inline]
    pub const fn dyes(self) -> &'static [Dye] {{
        use Dye::*;
        match self {{
            {method_dyes}
        }}
    }}

    /// Returns a color representing `self`. Does not necessarily correspond to a dye.
    #[must_use]
    #[inline]
    pub const fn color(self) -> Rgb {{
        match self {{
            {method_color}
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
            {method_short_names}
        }}
    }}
}}
