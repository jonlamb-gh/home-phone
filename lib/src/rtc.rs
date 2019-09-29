use core::fmt;

/// A DateTime wrapper over ds323x::DateTime
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateTime(ds323x::DateTime);

// TODO
//impl DateTime
// pub fn is_valid() ?

impl From<ds323x::DateTime> for DateTime {
    fn from(dt: ds323x::DateTime) -> Self {
        DateTime(dt)
    }
}

const DOW_STRINGS: [&'static str; 8] = ["Err", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

const MONTH_STRINGS: [&'static str; 13] = [
    "Err", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// Formats like chrono::DateTime "%a %b %e %H:%M:%S"
// Example: Sun Sep 29 07:10:15
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dow = match self.0.weekday {
            i @ 1..=7 => DOW_STRINGS[i as usize],
            _ => DOW_STRINGS[0],
        };
        let month = match self.0.month {
            i @ 1..=12 => MONTH_STRINGS[i as usize],
            _ => MONTH_STRINGS[0],
        };
        let day = self.0.day;
        let hour = match self.0.hour {
            ds323x::Hours::AM(hr) => hr,
            ds323x::Hours::PM(hr) => hr,
            ds323x::Hours::H24(hr) => hr,
        };
        let min = self.0.minute;
        let sec = self.0.second;

        write!(
            f,
            "{} {} {:02} {:02}:{:02}:{:02}",
            dow, month, day, hour, min, sec
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, trace};

    #[test_case]
    fn datetime_formating() {
        trace!("datetime_formating");
        let dt = DateTime(ds323x::DateTime {
            year: 0,
            month: 0,
            day: 0,
            weekday: 0,
            hour: ds323x::Hours::AM(0),
            minute: 0,
            second: 0,
        });
        debug!("{}", dt);
        let dt = DateTime(ds323x::DateTime {
            year: 2019,
            month: 1,
            day: 1,
            weekday: 1,
            hour: ds323x::Hours::H24(1),
            minute: 1,
            second: 1,
        });
        debug!("{}", dt);
    }
}
