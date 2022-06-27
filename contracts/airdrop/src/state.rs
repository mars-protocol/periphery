use cosmwasm_std::Addr;
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

pub const CONFIG: Item<Config> = Item::new("config");

pub const CLAIMED: Map<&str, bool> = Map::new("claimed");
