use crate::Rgb;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Snack {
    Apple,
    Pear,
    Berries,
    Plum,
    Fruit,
    Pineapple
}

impl Snack {
    pub const fn values() -> &'static [Snack] {
        &[
            Snack::Apple,
            Snack::Pear,
            Snack::Berries,
            Snack::Plum,
            Snack::Fruit,
            Snack::Pineapple
        ]
    }

    pub fn iter() -> impl Iterator<Item = Snack> {
        Snack::values().iter().copied()
    }

    pub const fn short_name(self) -> &'static str {
        match self {
            Snack::Apple     => "apple",
            Snack::Pear      => "pear",
            Snack::Berries   => "berries",
            Snack::Plum      => "plum",
            Snack::Fruit     => "fruit",
            Snack::Pineapple => "pineapple"
        }
    }

    pub const fn effect(self) -> (i8, i8, i8) {
        match self {
            Snack::Apple     => ( 5, -5, -5),
            Snack::Pear      => (-5,  5, -5),
            Snack::Berries   => (-5, -5,  5),
            Snack::Plum      => (-5,  5,  5),
            Snack::Fruit     => ( 5, -5,  5),
            Snack::Pineapple => ( 5,  5, -5)
        }
    }

    pub fn alter(self, color: Rgb) -> Option<Rgb> {
        let (r, g, b) = self.effect();

        color.checked_add_signed(r, g, b)
    }
}
