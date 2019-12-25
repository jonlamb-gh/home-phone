use crate::display::{Row, RowFormatter, RowStorage};
use crate::hal::time::{DisplayableInstant, Duration};
use crate::phone_number::PhoneNumber;
use crate::rtc::DateTime;
use core::fmt::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct InCallStateData {
    pub system_time: DateTime,
    pub remote: PhoneNumber,
    pub call_duration: Duration,
}

impl Default for InCallStateData {
    fn default() -> Self {
        InCallStateData {
            system_time: DateTime::default(),
            remote: PhoneNumber::default(),
            call_duration: Duration::default(),
        }
    }
}

impl RowFormatter for InCallStateData {
    fn format_row(&self, row: Row, storage: &mut RowStorage) -> Result<(), fmt::Error> {
        storage.clear();

        match row {
            Row::Zero => {
                // TODO - alignment doesn't seem to work with format_args here?
                write!(storage, "{: ^20}", self.remote)?;
            }
            Row::One => {
                // TODO - alignment doesn't seem to work with format_args here?
                write!(
                    storage,
                    "{: ^20}",
                    format_args!("Duration {}", DisplayableInstant::from(self.call_duration))
                )?;
            }
            Row::Two => {
                write!(storage, "{: ^20}", "")?;
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
        let data = InCallStateData::default();
        format_data(&data);
    }
}
