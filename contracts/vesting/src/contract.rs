#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::must_pay;

use crate::{
    error::{Error, Result},
    helpers::{compute_position_response, compute_withdrawable},
    migrations::{v1_1_0, v1_1_1},
    msg::{
        Config, ExecuteMsg, MigrateMsg, Position, PositionResponse, QueryMsg, Schedule,
        VotingPowerResponse,
    },
    state::{CONFIG, POSITIONS},
};

pub const CONTRACT_NAME: &str = "crates.io:mars-vesting";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    cfg: Config<String>,
) -> Result<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = cfg.check(deps.api)?;
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    let api = deps.api;
    match msg {
        ExecuteMsg::UpdateConfig {
            new_cfg,
        } => update_config(deps, info, new_cfg),
        ExecuteMsg::CreatePosition {
            user,
            vest_schedule,
        } => create_position(deps, info, api.addr_validate(&user)?, vest_schedule),
        ExecuteMsg::TerminatePosition {
            user,
        } => terminate_position(deps, env, info, api.addr_validate(&user)?),
        ExecuteMsg::Withdraw {} => withdraw(deps, env.block.time.seconds(), info.sender),
    }
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_cfg: Config<String>,
) -> Result<Response> {
    let cfg = CONFIG.load(deps.storage)?;

    // only owner can update config
    if info.sender != cfg.owner {
        return Err(Error::NotOwner);
    }

    let new_cfg = new_cfg.check(deps.api)?;
    CONFIG.save(deps.storage, &new_cfg)?;

    Ok(Response::new().add_attribute("action", "mars/vesting/update_config"))
}

pub fn create_position(
    deps: DepsMut,
    info: MessageInfo,
    user_addr: Addr,
    vest_schedule: Schedule,
) -> Result<Response> {
    let cfg = CONFIG.load(deps.storage)?;

    // only owner can create allocations
    if info.sender != cfg.owner {
        return Err(Error::NotOwner);
    }

    let total = must_pay(&info, &cfg.denom)?;

    POSITIONS.update(deps.storage, &user_addr, |position| {
        if position.is_some() {
            return Err(Error::PositionExists);
        }
        Ok(Position {
            total,
            vest_schedule: vest_schedule.clone(),
            withdrawn: Uint128::zero(),
        })
    })?;

    Ok(Response::new()
        .add_attribute("action", "mars/vesting/position_created")
        .add_attribute("user", user_addr)
        .add_attribute("total", total)
        .add_attribute("start_time", vest_schedule.start_time.to_string())
        .add_attribute("cliff", vest_schedule.cliff.to_string())
        .add_attribute("duration", vest_schedule.duration.to_string()))
}

pub fn terminate_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_addr: Addr,
) -> Result<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // only owner can terminate allocations
    if info.sender != cfg.owner {
        return Err(Error::NotOwner);
    }

    let mut position = POSITIONS.load(deps.storage, &user_addr)?;

    let (vested, _, _) = compute_withdrawable(
        current_time,
        position.total,
        position.withdrawn,
        &position.vest_schedule,
        &cfg.unlock_schedule,
    );

    // unvested tokens are to be reclaimed by the owner
    let reclaim = position.total - vested;

    // set position total amount to be the vested amount so far, and vesting end time to now
    position.total = vested;
    position.vest_schedule.duration = current_time - position.vest_schedule.start_time;
    POSITIONS.save(deps.storage, &user_addr, &position)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: cfg.owner.into(),
            amount: coins(reclaim.u128(), cfg.denom),
        }))
        .add_attribute("action", "mars/vesting/terminate_position")
        .add_attribute("user", user_addr)
        .add_attribute("vested", vested)
        .add_attribute("relaimed", reclaim))
}

pub fn withdraw(deps: DepsMut, time: u64, user_addr: Addr) -> Result<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let mut position = POSITIONS.load(deps.storage, &user_addr)?;

    let (_, _, withdrawable) = compute_withdrawable(
        time,
        position.total,
        position.withdrawn,
        &position.vest_schedule,
        &cfg.unlock_schedule,
    );

    if withdrawable.is_zero() {
        return Err(Error::ZeroWithdrawable);
    }

    position.withdrawn += withdrawable;
    POSITIONS.save(deps.storage, &user_addr, &position)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: user_addr.to_string(),
            amount: coins(withdrawable.u128(), cfg.denom),
        }))
        .add_attribute("action", "mars/vesting/withdraw")
        .add_attribute("user", user_addr)
        .add_attribute("timestamp", time.to_string())
        .add_attribute("withdrawable", withdrawable))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    let api = deps.api;
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::VotingPower {
            user,
        } => to_binary(&query_voting_power(deps, api.addr_validate(&user)?)?),
        QueryMsg::VotingPowers {
            start_after,
            limit,
        } => to_binary(&query_voting_powers(deps, start_after, limit)?),
        QueryMsg::Position {
            user,
        } => to_binary(&query_position(deps, env.block.time.seconds(), api.addr_validate(&user)?)?),
        QueryMsg::Positions {
            start_after,
            limit,
        } => to_binary(&query_positions(deps, env.block.time.seconds(), start_after, limit)?),
    }
    .map_err(Into::into)
}

pub fn query_config(deps: Deps) -> Result<Config<String>> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(cfg.into())
}

pub fn query_voting_power(deps: Deps, user_addr: Addr) -> Result<VotingPowerResponse> {
    let voting_power = match POSITIONS.may_load(deps.storage, &user_addr) {
        Ok(Some(position)) => position.total - position.withdrawn,
        Ok(None) => Uint128::zero(),
        Err(err) => return Err(err.into()),
    };

    Ok(VotingPowerResponse {
        user: user_addr.to_string(),
        voting_power,
    })
}

pub fn query_position(deps: Deps, time: u64, user_addr: Addr) -> Result<PositionResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let position = POSITIONS.load(deps.storage, &user_addr)?;

    Ok(compute_position_response(time, user_addr, &position, &cfg.unlock_schedule))
}

pub fn query_voting_powers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<VotingPowerResponse>> {
    let addr: Addr;
    let start = match &start_after {
        Some(addr_str) => {
            addr = deps.api.addr_validate(addr_str)?;
            Some(Bound::exclusive(&addr))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (user_addr, position) = res?;
            Ok(VotingPowerResponse {
                user: user_addr.to_string(),
                voting_power: position.total - position.withdrawn,
            })
        })
        .collect()
}

pub fn query_positions(
    deps: Deps,
    time: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<PositionResponse>> {
    let cfg = CONFIG.load(deps.storage)?;

    let addr: Addr;
    let start = match &start_after {
        Some(addr_str) => {
            addr = deps.api.addr_validate(addr_str)?;
            Some(Bound::exclusive(&addr))
        }
        None => None,
    };

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    POSITIONS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (user_addr, position) = res?;
            Ok(compute_position_response(time, user_addr, &position, &cfg.unlock_schedule))
        })
        .collect()
}

//--------------------------------------------------------------------------------------------------
// Migration
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _: Env, msg: MigrateMsg) -> Result<Response> {
    match msg {
        MigrateMsg::V1_0_0ToV1_1_0 {} => v1_1_0::migrate(deps),
        MigrateMsg::V1_1_0ToV1_1_1(updates) => v1_1_1::migrate(deps, updates),
    }
}
