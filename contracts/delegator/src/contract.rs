use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use mars_types::MarsMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::{execute, query};

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
    execute::init(deps, msg.bond_denom)
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response<MarsMsg>, ContractError> {
    match msg {
        SudoMsg::Bond {
            validators,
            ending_time,
        } => execute::bond(deps, env, validators, ending_time),
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
