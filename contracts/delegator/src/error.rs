use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("contract does not hold any coin to be bonded")]
    NothingToBond,

    #[error("contract does not hold any coin to be refunded")]
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
