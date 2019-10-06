use crate::display::{Row, RowFormatter, RowStorage};
use crate::rtc::DateTime;
use core::fmt::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct IdleStateData {
    // TODO - storage for logs
    pub missed_calls: usize,

    pub system_time: DateTime,

    // TODO row storage or heapless::String?
    // row storage is fixed capacity, String could be any and truncated
    pub message: Option<RowStorage>,
}

impl Default for IdleStateData {
    fn default() -> Self {
        IdleStateData {
            missed_calls: 0,
            system_time: DateTime::default(),
            message: None,
        }
    }
}

impl RowFormatter for IdleStateData {
    fn format_row(&self, row: Row, storage: &mut RowStorage) -> Result<(), fmt::Error> {
        storage.clear();

        match row {
            Row::Zero => {
                if self.missed_calls != 0 {
                    write!(storage, "{: ^20}", "'*' Next | Clear '#'")?;
                }
            }
            Row::One => {
                // TODO - alignment doesn't seem to work with format_args here
                if self.missed_calls != 0 {
                    write!(
                        storage,
                        "{: ^20}",
                        format_args!("{} Missed Calls", self.missed_calls)
                    )?;
                }
            }
            Row::Two => {
                if let Some(msg) = &self.message {
                    write!(storage, "{: ^20}", msg.as_str())?;
                } else {
                    write!(storage, "{: ^20}", "")?;
                }
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
    use log::{debug, trace};

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

    #[test_case]
    fn default_formatter() {
        trace!("default_formatter");
        let data = IdleStateData::default();
        format_data(&data);
    }

    #[test_case]
    fn missed_calls_formatter() {
        trace!("missed_calls_formatter");
        let data = IdleStateData {
            missed_calls: 2,
            system_time: DateTime::default(),
            message: None,
        };
        format_data(&data);
    }

    #[test_case]
    fn with_message_formatter() {
        trace!("with_message_formatter");
        let data = IdleStateData {
            missed_calls: 0,
            system_time: DateTime::default(),
            message: Some(RowStorage::from("A message")),
        };
        format_data(&data);
    }
}
