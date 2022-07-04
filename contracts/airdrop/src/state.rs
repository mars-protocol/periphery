use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

/// Root of the Merkle tree, in base64 encoding; each leaf is a string with the format `{recipient},{amount}`
pub const ROOT: Item<String> = Item::new("merkle_root");

/// UNIX timestamp after which unclaimed tokens can be transferred to the community pool
pub const DEADLINE: Item<u64> = Item::new("claim_deadline");

/// The amount of tokens each user has claimed
///
/// NOTE: The key is `&str` instead of `Addr` because i am unsure whether `deps.api` can validate terra
/// addresses when the chain's account prefix is `mars`. Maybe it can, idk
pub const CLAIMED: Map<&str, Uint128> = Map::new("claimed");
