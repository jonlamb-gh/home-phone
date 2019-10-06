mod idle_state_data;

pub use crate::phone_state::idle_state_data::IdleStateData;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PhoneState {
    Idle,
    CallPending,
    InCall,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhoneStateData {
    IdleStateData(IdleStateData),
}
