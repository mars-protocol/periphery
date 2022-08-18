use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Root of the Merkle tree, in hex encoding; each leaf is the SHA256 hash of the string `{recipient},{amount}`
    pub merkle_root: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Claim an airdrop
    ///
    /// Parameters:
    /// - `terra_acct_pk`: Public key of the Mars Classic token holder, in hex encoding
    /// - `mars_acct`: Mars address to which the claimed tokens shall to sent
    /// - `amount`: Amount of Mars tokens claim
    /// - `proof`: Proof that leaf `{terra-acct}:{amount}` exists in the Merkle tree, in hex encoding
    /// - `signature`: Signature produced by signing message `airdrop for {terra-acct} of {amount} umars
    ///   shall be released to {mars-acct}` by the Terra account's private key, in hex encoding
    Claim {
        terra_acct_pk: String,
        mars_acct: String,
        amount: Uint128,
        proof: Vec<String>,
        signature: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    /// Reclaim unclaimed tokens to the community pool
    Clawback {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The contract's config; returns `ConfigResponse`
    Config {},
    /// The amount of tokens that an account has claimed; returns `ClaimedResponse`
    Claimed {
        terra_acct: String,
    },
    /// Enumerate all accounts that have claimed; returns `Vec<ClaimedResponse>`
    AllClaimed {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Verify the validity of a signature; returns `bool`
    VerifySignature {
        terra_acct_pk: String,
        mars_acct: String,
        amount: Uint128,
        signature: String,
    },
    /// Verify a Merkle proof; returns `bool`
    VerifyProof {
        terra_acct: String,
        amount: Uint128,
        proof: Vec<String>,
    },
}

pub type ConfigResponse = InstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimedResponse {
    /// The user's Terra account address
    pub terra_acct: String,
    /// The amount of tokens this user has claimed
    pub amount: Uint128,
}

/// Generate the message that needs to be signed by the Terra account's private key
pub fn msg(terra_acct: &str, mars_acct: &str, amount: Uint128) -> String {
    format!("airdrop for {} of {} umars shall be released to {}", terra_acct, amount, mars_acct)
}

/// Generate the leaf of the Merkle tree
///
/// NOTE: The actual leaf used in the Merkle tree is SHA256 hash of this string
pub fn leaf(terra_acct: &str, amount: Uint128) -> String {
    format!("{}:{}", terra_acct, amount)
}
