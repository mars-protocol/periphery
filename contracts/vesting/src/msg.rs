use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

/// Denomination of the token to be vested
pub const VEST_DENOM: &str = "umars";

#[cw_serde]
pub struct Schedule {
    /// Time when vesting/unlocking starts
    pub start_time: u64,
    /// Time before with no token is to be vested/unlocked
    pub cliff: u64,
    /// Duration of the vesting/unlocking process. At time `start_time + duration`, the tokens are
    /// vested/unlocked in full
    pub duration: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// The contract's owner
    pub owner: String,
    /// Schedule for token unlocking; this schedule is the same for all users
    pub unlock_schedule: Schedule,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new vesting position for a user
    CreatePosition {
        user: String,
        vest_schedule: Schedule,
    },
    /// Withdraw vested and unlocked MARS tokens
    Withdraw {},
    /// Transfer the contract's ownership to another account
    TransferOwnership(String),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract's configurations
    #[returns(ConfigResponse)]
    Config {},
    /// Amount of MARS tokens of a vesting recipient current locked in the contract
    #[returns(VotingPowerResponse)]
    VotingPower {
        user: String,
    },
    /// Enumerate all vesting recipients and return their current voting power
    #[returns(Vec<VotingPowerResponse>)]
    VotingPowers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Details of a recipient's vesting position
    ///
    /// NOTE: This query depends on block time, therefore it may not work with time travel queries.
    /// In such cases, use WASM raw query instead.
    #[returns(PositionResponse)]
    Position {
        user: String,
    },
    /// Enumerate all vesting positions
    ///
    /// NOTE: This query depends on block time, therefore it may not work with time travel queries.
    /// In such cases, use WASM raw query instead.
    #[returns(Vec<PositionResponse>)]
    Positions {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

pub type ConfigResponse = InstantiateMsg;

#[cw_serde]
pub struct VotingPowerResponse {
    /// Address of the user
    pub user: String,
    /// The user's current voting power, i.e. the amount of MARS tokens locked in vesting contract
    pub voting_power: Uint128,
}

#[cw_serde]
pub struct PositionResponse {
    /// Address of the user
    pub user: String,
    /// Total amount of MARS tokens allocated to this recipient
    pub total: Uint128,
    /// Amount of tokens that have been vested, according to the vesting schedule
    pub vested: Uint128,
    /// Amount of tokens that have been unlocked, according to the unlocking schedule
    pub unlocked: Uint128,
    /// Amount of tokens that have already been withdrawn
    pub withdrawn: Uint128,
    /// Amount of tokens that can be withdrawn now, defined as the smaller of vested and unlocked amounts,
    /// minus the amount already withdrawn
    pub withdrawable: Uint128,
    /// This vesting position's vesting schedule
    pub vest_schedule: Schedule,
}
