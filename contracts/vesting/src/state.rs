use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::Schedule;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    /// Total amount of MARS allocated
    pub total: Uint128,
    /// Amount of MARS already withdrawn
    pub withdrawn: Uint128,
    /// The user's vesting schedule
    pub vest_schedule: Schedule,
}

pub const OWNER: Item<Addr> = Item::new("owner");
pub const PENDING_OWNER: Item<Addr> = Item::new("pending_owner");
pub const UNLOCK_SCHEDULE: Item<Schedule> = Item::new("unlock_schedule");
pub const TOTAL_VOTING_POWER: Item<Uint128> = Item::new("total_voting_power");
pub const POSITIONS: Map<&Addr, Position> = Map::new("positions");
