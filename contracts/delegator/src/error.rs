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

    #[error("incorrect contract: expecting {expect}, found {found}")]
    IncorrectContract {
        expect: String,
        found: String,
    },

    #[error("incorrect version: expecting {expect}, found {found}")]
    IncorrectVersion {
        expect: String,
        found: String,
    },
}

impl ContractError {
    pub fn ending_time_not_reached(ending_time: u64, current_time: u64) -> Self {
        Self::EndingTimeNotReached {
            ending_time,
            current_time,
        }
    }

    pub fn incorrect_contract(expect: impl Into<String>, found: impl Into<String>) -> Self {
        Self::IncorrectContract {
            expect: expect.into(),
            found: found.into(),
        }
    }

    pub fn incorrect_version(expect: impl Into<String>, found: impl Into<String>) -> Self {
        Self::IncorrectVersion {
            expect: expect.into(),
            found: found.into(),
        }
    }
}
