#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Row {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl Row {
    pub fn enumerate() -> &'static [Self] {
        &[Row::Zero, Row::One, Row::Two, Row::Three]
    }
}
