use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use crate::{
    error::ContractError,
    execute, migrations,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    query,
    types::MarsMsg,
};

pub const CONTRACT_NAME: &str = "crates.io:mars-delegator";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    execute::init(deps, msg)
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response<MarsMsg>, ContractError> {
    match msg {
        SudoMsg::Bond {} => execute::bond(deps, env),
        SudoMsg::ForceUnbond {} => execute::force_unbond(deps, env),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<MarsMsg>, ContractError> {
    match msg {
        ExecuteMsg::Unbond {} => execute::unbond(deps, env),
        ExecuteMsg::Refund {} => execute::refund(deps, env),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::query_config(deps)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    migrations::v1_1::migrate(deps)
}
