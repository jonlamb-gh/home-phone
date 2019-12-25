use crate::display::{Row, RowFormatter, RowStorage};
use crate::phone_number::PhoneNumber;
use crate::rtc::DateTime;
use core::fmt::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct CallPendingStateData {
    pub system_time: DateTime,
    pub remote: PhoneNumber,
}

impl Default for CallPendingStateData {
    fn default() -> Self {
        CallPendingStateData {
            system_time: DateTime::default(),
            remote: PhoneNumber::default(),
        }
    }
}

impl RowFormatter for CallPendingStateData {
    fn format_row(&self, row: Row, storage: &mut RowStorage) -> Result<(), fmt::Error> {
        storage.clear();

        match row {
            Row::Zero => {
                write!(storage, "{: ^20}", "'*' Decl | Ans '#'")?;
            }
            Row::One => {
                write!(storage, "{: ^20}", "Incoming Call")?;
            }
            Row::Two => {
                // TODO - alignment doesn't seem to work with format_args here?
                write!(storage, "{: ^20}", self.remote)?;
            }
            Row::Three => {
                write!(storage, "{: ^20}", self.system_time)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    fn format_data<T: RowFormatter>(data: &T) {
        let mut storage = RowStorage::new();
        debug!("**********************");
        for row in Row::enumerate() {
            data.format_row(*row, &mut storage).unwrap();
            debug!("*{:20}*", storage.as_str());
            assert!(storage.len() <= 21);
        }
        debug!("**********************");
    }

    #[test]
    fn default_formatter() {
        let data = CallPendingStateData::default();
        format_data(&data);
    }
}
