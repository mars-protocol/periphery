use cosmwasm_std::{coins, BankMsg, CosmosMsg, DepsMut, Env, Response};
use cw2::set_contract_version;

use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::Result,
    msg::V1_1_2Updates,
    state::{CONFIG, WITHDRAW_ENABLED},
};

const FROM_VERSION: &str = "1.1.1";

pub fn migrate(deps: DepsMut, env: Env, msg: V1_1_2Updates) -> Result<Response> {
    // Make sure we're migrating the correct contract and from the correct version
    cw2::assert_contract_version(deps.as_ref().storage, CONTRACT_NAME, FROM_VERSION)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Disable withdraws during migration
    WITHDRAW_ENABLED.save(deps.storage, &false)?;

    // Query mars balance of the contract
    let cfg = CONFIG.load(deps.storage)?;
    let mars_amount = deps.querier.query_balance(env.contract.address, &cfg.denom)?.amount;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: msg.mars_receiver_addr,
            amount: coins(mars_amount.u128(), cfg.denom),
        }))
        .add_attribute("action", "migrate")
        .add_attribute("from_version", FROM_VERSION)
        .add_attribute("to_version", CONTRACT_VERSION))
}
