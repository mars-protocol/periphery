use cosmwasm_schema::{cw_serde, QueryResponses};

pub const BOND_DENOM: &str = "umars";

#[cw_serde]
pub struct InstantiateMsg {
    /// The ending time for the delegation program, as UNIX timestamp.
    ///
    /// Once this time has elapsed, anyone can invoke the `unbond` method to unbond the delegations.
    ///
    /// Additionally, Mars Hub governance can decide to prematurely end the delegation program if
    /// they see fit, ignoring the ending time, by invoking the `force_unbond` sudo message.
    pub ending_time: u64,
}

#[cw_serde]
pub enum SudoMsg {
    /// Forcibly unbond the delegations.
    ///
    /// This "sudo" message can only be invoked by the gov module, and ignores whether the
    /// `ending_time` has been reached.
    ForceUnbond {},
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Unbond the delegations.
    ///
    /// Can be invoked by anyone after `ending_time` is reached.
    Unbond {},

    /// Donate all coins held by the contract to the community pool.
    Refund {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The ending time for the delegation program.
    #[returns(u64)]
    EndingTime {},
}
