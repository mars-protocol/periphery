use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Root of the Merkle tree, in base64 encoding; each leaf is a string with the format `{recipient},{amount}`
    pub merkle_root: String,
    /// UNIX timestamp after which unclaimed tokens can be transferred to the community pool
    pub claim_deadline: u64,
    /// Address of the community pool
    pub community_pool: Addr,
}

/// The contract's configurations
pub const CONFIG: Item<Config> = Item::new("config");

/// The amount of tokens each user has claimed
///
/// NOTE: The key is `&str` instead of `Addr` because i am unsure whether `deps.api` can validate terra
/// addresses when the chain's account prefix is `mars`. Maybe it can, idk
pub const CLAIMED: Map<&str, Uint128> = Map::new("claimed");
