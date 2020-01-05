use core::fmt;

pub use core::time::Duration;

pub type Instant = core::time::Duration;

pub struct DisplayableInstant(pub Instant);

impl From<Instant> for DisplayableInstant {
    fn from(i: Instant) -> DisplayableInstant {
        DisplayableInstant(i)
    }
}

impl fmt::Display for DisplayableInstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0.as_secs(), self.0.subsec_nanos())
    }
}
