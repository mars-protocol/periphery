use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, CustomMsg};

#[cw_serde]
pub enum MarsMsg {
    /// This is translated to a [MsgFundCommunityPool](https://github.com/cosmos/cosmos-sdk/blob/v0.45.6/proto/cosmos/distribution/v1beta1/tx.proto#L67-L76).
    /// `depositor` is automatically filled with the current contract's address.
    FundCommunityPool {
        amount: Vec<Coin>,
    },
}

impl CustomMsg for MarsMsg {}
