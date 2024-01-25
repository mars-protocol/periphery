use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Api, StdResult, Uint128};

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
pub struct Position {
    /// Total amount of MARS allocated
    pub total: Uint128,
    /// Amount of MARS already withdrawn
    pub withdrawn: Uint128,
    /// The user's vesting schedule
    pub vest_schedule: Schedule,
}

#[cw_serde]
pub struct Config<T> {
    /// The contract's owner
    pub owner: T,
    /// Denomination of the token to be vested
    pub denom: String,
    /// Schedule for token unlocking; this schedule is the same for all users
    pub unlock_schedule: Schedule,
}

impl Config<String> {
    pub fn check(self, api: &dyn Api) -> StdResult<Config<Addr>> {
        Ok(Config {
            owner: api.addr_validate(&self.owner)?,
            denom: self.denom,
            unlock_schedule: self.unlock_schedule,
        })
    }
}

impl From<Config<Addr>> for Config<String> {
    fn from(cfg: Config<Addr>) -> Self {
        Config {
            owner: cfg.owner.into(),
            denom: cfg.denom,
            unlock_schedule: cfg.unlock_schedule,
        }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update the contract's configurations
    UpdateConfig {
        new_cfg: Config<String>,
    },
    /// Create a new vesting position for a user
    CreatePosition {
        user: String,
        vest_schedule: Schedule,
    },
    /// Terminate a vesting position, collect all unvested tokens
    TerminatePosition {
        user: String,
    },
    /// Withdraw vested and unlocked MARS tokens
    Withdraw {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract's configurations
    #[returns(Config<String>)]
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

#[cw_serde]
pub enum MigrateMsg {
    V1_0_0ToV1_1_0 {},
    V1_1_0ToV1_1_1(V1_1_1Updates),
}

#[cw_serde]
pub struct V1_1_1Updates {
    /// Array of positions for alteration
    pub position_alterations: Vec<PositionAlteration>,
    /// Total amount of MARS to be reclaimed
    pub total_reclaim: Uint128,
}

#[cw_serde]
pub struct PositionAlteration {
    /// Address of user to alter
    pub addr: Addr,
    /// Total amount of MARS allocated previously
    pub total_old: Uint128,
    /// Total amount of MARS allocated now
    pub total_new: Uint128,
    /// Total amount of MARS to be reclaimed
    pub reclaim: Uint128,
}
