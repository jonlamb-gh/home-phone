mod call_pending_state_data;
mod idle_state_data;
mod in_call_state_data;

pub use crate::phone_state::call_pending_state_data::CallPendingStateData;
pub use crate::phone_state::idle_state_data::IdleStateData;
pub use crate::phone_state::in_call_state_data::InCallStateData;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PhoneState {
    Idle,
    CallPending,
    InCall,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhoneStateData {
    Idle(IdleStateData),
    CallPending(CallPendingStateData),
    InCall(InCallStateData),
}
