use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

/// Global configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// Contract owner
    pub owner: Addr,
    /// Address provider returns addresses for all protocol contracts
    pub address_provider: Addr,
}

/// Liquidate under-collateralized native loans. Coins used to repay must be sent in the
/// transaction this call is made.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Liquidate {
    /// Denom of the collateral asset, which liquidator gets from the borrower
    pub collateral_denom: String,
    /// Denom of the debt asset
    pub debt_denom: String,
    /// The address of the borrower getting liquidated
    pub user_address: String,
}
