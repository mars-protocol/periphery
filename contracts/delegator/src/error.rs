use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("contract does not hold any coin")]
    NothingToRefund,

    #[error("ending time is not reached yet! ending: {ending_time}, current: {current_time}")]
    EndingTimeNotReached {
        ending_time: u64,
        current_time: u64,
    },
}

impl ContractError {
    pub fn ending_time_not_reached(ending_time: u64, current_time: u64) -> Self {
        Self::EndingTimeNotReached {
            ending_time,
            current_time,
        }
    }
}
