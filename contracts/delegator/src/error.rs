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

    #[error("validator with address `{address}` does not exist")]
    ValidatorNotFound {
        address: String,
    },

    #[error("invalid ending time `{ending_time}`: must be later than current time {current_time}")]
    InvalidEndingTime {
        ending_time: u64,
        current_time: u64,
    },

    #[error("ending time is not reached yet! ending: {ending_time}, current: {current_time}")]
    EndingTimeNotReached {
        ending_time: u64,
        current_time: u64,
    },
}
