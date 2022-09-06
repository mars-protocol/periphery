use crate::types::Liquidate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    /// Contract owner
    pub owner: String,
    /// Address provider returns addresses for all protocol contracts
    pub address_provider: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Set emission per second for an asset to holders of its maToken
    LiquidateMany {
        liquidations: Vec<Liquidate>,
    },

    /// Update contract config (only callable by owner)
    UpdateConfig {
        owner: Option<String>,
        address_provider: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query contract config
    Config {},
}
