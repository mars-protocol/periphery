#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    Payment(#[from] cw_utils::PaymentError),

    #[error("caller is not owner")]
    NotOwner,

    #[error("a vesting position already exists for this user")]
    PositionExists,

    #[error("withdrawable amount is zero")]
    ZeroWithdrawable,

    #[error("{0}")]
    Version(#[from] cw2::VersionError),

    #[error("{0}")]
    Overflow(#[from] cosmwasm_std::OverflowError),
}

pub(crate) type Result<T> = core::result::Result<T, Error>;
