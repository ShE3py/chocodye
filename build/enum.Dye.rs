/// A color that can be found as the plumage of a chocobo.
///
/// Some dyes, such as vanilla yellow, are not included in this enum.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Dye {{
{variants}
}}

impl Dye {{
    /// Contains all eighty-five `Dye` variants.
    pub const VALUES: [Dye; 85] = [
        {associatedconstant_VALUES}
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
            {method_category}
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
            {method_color}
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
            {method_short_names}
        }}
    }}
}}
