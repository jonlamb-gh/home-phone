use crate::display::{Row, RowStorage};
use core::fmt;

pub trait RowFormatter {
    fn format_row(&mut self, row: Row, storage: &mut RowStorage) -> Result<(), fmt::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;
    use log::{debug, trace};

    struct TestData {
        r0: usize,
        r1: f32,
        r2: &'static str,
        r3: i8,
    }

    impl RowFormatter for TestData {
        fn format_row(&mut self, row: Row, storage: &mut RowStorage) -> Result<(), fmt::Error> {
            storage.clear();

            match row {
                Row::Zero => {
                    write!(storage, "{: ^20}", self.r0)?;
                }
                Row::One => {
                    write!(storage, "{: <20}", self.r1)?;
                }
                Row::Two => {
                    write!(storage, "{: ^20}", self.r2)?;
                }
                Row::Three => {
                    write!(storage, "{: >20}", self.r3)?;
                }
            }
            Ok(())
        }
    }

    #[test_case]
    fn row_formatter() {
        trace!("row_formatter");
        let mut data = TestData {
            r0: 1234,
            r1: -23.34,
            r2: "Up to 20 characters.",
            r3: -123,
        };
        let mut storage = RowStorage::new();
        debug!("**********************");
        for row in Row::enumerate() {
            data.format_row(*row, &mut storage).unwrap();
            debug!("*{}*", storage.as_str());
            assert!(storage.len() <= 21);
        }
        debug!("**********************");
    }
}
