#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, QuerierWrapper, Response, StdResult, WasmMsg,
};
use mars_outpost::address_provider::MarsContract;
use mars_outpost::{address_provider, red_bank};
use std::collections::HashMap;
use std::ops::SubAssign;

use mars_outpost::error::MarsError;
use mars_outpost::helpers::option_string_to_addr;

use mars_outpost::red_bank::UserHealthStatus;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;
use crate::types::{Config, Liquidate};

pub const CONTRACT_NAME: &str = "crates.io:mars-liquidation-filterer";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// INIT

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        address_provider: deps.api.addr_validate(&msg.address_provider)?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

// HANDLERS

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            address_provider,
        } => Ok(execute_update_config(deps, info, owner, address_provider)?),
        ExecuteMsg::LiquidateMany {
            liquidations,
        } => execute_liquidate(deps, info, &env.contract.address, liquidations),
        ExecuteMsg::Refund {
            recipient,
        } => execute_refund(&deps.querier, &env.contract.address, &recipient),
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    address_provider: Option<String>,
) -> Result<Response, MarsError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(MarsError::Unauthorized {});
    };

    config.owner = option_string_to_addr(deps.api, owner, config.owner)?;
    config.address_provider =
        option_string_to_addr(deps.api, address_provider, config.address_provider)?;

    CONFIG.save(deps.storage, &config)?;

    let response =
        Response::new().add_attribute("action", "periphery/liquidation-filterer/update_config");

    Ok(response)
}

fn execute_liquidate(
    deps: DepsMut,
    info: MessageInfo,
    contract: &Addr,
    liquidations: Vec<Liquidate>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let red_bank_addr = address_provider::helpers::query_address(
        deps.as_ref(),
        &config.address_provider,
        MarsContract::RedBank,
    )?;

    // There shouldn't be duplicated denoms.
    // The amount for a denom should be equal or greater than sum of all amounts from liquidate messages for the same denom.
    let mut funds: HashMap<_, _> = info.funds.into_iter().map(|c| (c.denom, c.amount)).collect();

    let mut messages = vec![];
    for liquidate in liquidations {
        let user_position_response =
            query_user_position(deps.as_ref(), &red_bank_addr, &liquidate.user_address)?;

        if let UserHealthStatus::Borrowing {
            liq_threshold_hf,
            ..
        } = user_position_response.health_status
        {
            if liq_threshold_hf < Decimal::one() {
                // Check if there are enough funds sent to cover all liquidations
                match funds.get_mut(&liquidate.debt_denom) {
                    Some(amount) if *amount >= liquidate.amount => {
                        amount.sub_assign(liquidate.amount)
                    }
                    Some(_) => {
                        return Err(ContractError::InvalidFunds {
                            reason: format!("not enough {}", liquidate.debt_denom),
                        })
                    }
                    None => {
                        return Err(ContractError::InvalidFunds {
                            reason: format!("missing {}", liquidate.debt_denom),
                        })
                    }
                }

                let liq_msg = to_red_bank_liquidate_msg(&red_bank_addr, &liquidate)?;
                messages.push(liq_msg);
            }
        }
    }

    let refund_msg = WasmMsg::Execute {
        contract_addr: contract.to_string(),
        msg: to_binary(&ExecuteMsg::Refund {
            recipient: info.sender.to_string(),
        })?,
        funds: vec![],
    };

    let response = Response::new()
        .add_attributes(vec![attr("action", "periphery/liquidation-filterer/liquidate_many")])
        .add_messages(messages)
        .add_message(refund_msg);

    Ok(response)
}

fn execute_refund(
    querier: &QuerierWrapper,
    contract: &Addr,
    recipient: &str,
) -> Result<Response, ContractError> {
    let coins = querier.query_all_balances(contract)?;

    if coins.is_empty() {
        return Ok(Response::new());
    }

    let coins_str = coins.iter().map(|coin| coin.to_string()).collect::<Vec<_>>().join(",");

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: coins,
        })
        .add_attribute("action", "periphery/liquidation-filterer/refund")
        .add_attribute("recipient", recipient)
        .add_attribute("coins", coins_str))
}

fn query_user_position(
    deps: Deps<impl cosmwasm_std::CustomQuery>,
    red_bank_addr: &Addr,
    user: &str,
) -> StdResult<red_bank::UserPositionResponse> {
    let res: red_bank::UserPositionResponse = deps.querier.query_wasm_smart(
        red_bank_addr,
        &red_bank::QueryMsg::UserPosition {
            user: user.to_string(),
        },
    )?;

    Ok(res)
}

fn to_red_bank_liquidate_msg(red_bank_addr: &Addr, liquidate: &Liquidate) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: red_bank_addr.into(),
        msg: to_binary(&red_bank::ExecuteMsg::Liquidate {
            collateral_denom: liquidate.collateral_denom.clone(),
            user: liquidate.user_address.clone(),
        })?,
        funds: vec![coin(liquidate.amount.u128(), liquidate.debt_denom.clone())],
    }))
}

// QUERIES

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
