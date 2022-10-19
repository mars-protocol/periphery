use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct Config {
    /// Denomination of the coin that will be staked.
    pub bond_denom: String,

    /// The ending time for the delegation program, as UNIX timestamp.
    ///
    /// Once this time has elapsed, anyone can invoke the `unbond` method to unbond the delegations.
    ///
    /// Additionally, Mars Hub governance can decide to prematurely end the delegation program if
    /// they see fit, ignoring the ending time, by invoking the `force_unbond` sudo message.
    pub ending_time: u64,
}

pub type InstantiateMsg = Config;

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
    /// Return the contract configuration.
    #[returns(Config)]
    Config {},
}
