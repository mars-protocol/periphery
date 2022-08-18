#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::crypto::{pubkey_to_addr, verify_proof, verify_signature};
use crate::msg::{
    leaf, msg, ClaimedResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg,
};
use crate::state::{CLAIMED, ROOT};
use crate::types::MarsMsg;

const CONTRACT_NAME: &str = "crates.io:mars-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ROOT.save(deps.storage, &msg.merkle_root)?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<MarsMsg>> {
    let api = deps.api;
    match msg {
        ExecuteMsg::Claim {
            terra_acct_pk,
            mars_acct,
            amount,
            proof,
            signature,
        } => claim(deps, terra_acct_pk, api.addr_validate(&mars_acct)?, amount, proof, signature),
    }
}

pub fn claim(
    deps: DepsMut,
    terra_acct_pk: String,
    mars_acct: Addr,
    amount: Uint128,
    proof: Vec<String>,
    signature: String,
) -> StdResult<Response<MarsMsg>> {
    let root = ROOT.load(deps.storage)?;

    let terra_acct = pubkey_to_addr(&terra_acct_pk, "terra")?;

    // The Terra account must not have already claimed
    CLAIMED.update(deps.storage, &terra_acct, |claimed| {
        if claimed.is_some() {
            return Err(StdError::generic_err("account has already claimed"));
        }
        Ok(amount)
    })?;

    // The signature must be valid
    let msg = msg(&terra_acct, &mars_acct.to_string(), amount);
    if !verify_signature(deps.api, &msg, &terra_acct_pk, &signature)? {
        return Err(StdError::generic_err("invalid signature"));
    }

    // The Merkle proof must be valid
    let leaf = leaf(&terra_acct, amount);
    if !verify_proof(&leaf, &root, &proof)? {
        return Err(StdError::generic_err("invalid proof"));
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: mars_acct.to_string(),
            amount: coins(amount.u128(), "umars"),
        }))
        .add_attribute("action", "mars/airdrop/claim")
        .add_attribute("terra_acct", terra_acct)
        .add_attribute("mars_acct", mars_acct)
        .add_attribute("amount", amount))
}

//--------------------------------------------------------------------------------------------------
// Sudo
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> StdResult<Response<MarsMsg>> {
    match msg {
        SudoMsg::Clawback {} => clawback(deps, env),
    }
}

pub fn clawback(deps: DepsMut, env: Env) -> StdResult<Response<MarsMsg>> {
    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    let amount_str = amount.iter().map(|coin| coin.to_string()).collect::<Vec<_>>().join(",");

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(MarsMsg::FundCommunityPool {
            amount,
        }))
        .add_attribute("action", "mars/airdrop/clawback")
        .add_attribute("timestamp", env.block.time.seconds().to_string())
        .add_attribute("amount", amount_str))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Claimed {
            terra_acct,
        } => to_binary(&query_claimed(deps, terra_acct)?),
        QueryMsg::AllClaimed {
            start_after,
            limit,
        } => to_binary(&query_all_claimed(deps, start_after, limit)?),
        QueryMsg::VerifySignature {
            terra_acct_pk,
            mars_acct,
            amount,
            signature,
        } => to_binary(&query_verify_signature(deps, terra_acct_pk, mars_acct, amount, signature)?),
        QueryMsg::VerifyProof {
            terra_acct,
            amount,
            proof,
        } => to_binary(&query_verify_proof(deps, terra_acct, amount, proof)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        merkle_root: ROOT.load(deps.storage)?,
    })
}

pub fn query_claimed(deps: Deps, terra_acct: String) -> StdResult<ClaimedResponse> {
    Ok(ClaimedResponse {
        amount: CLAIMED.load(deps.storage, &terra_acct).unwrap_or_else(|_| Uint128::zero()),
        terra_acct,
    })
}

pub fn query_all_claimed(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<ClaimedResponse>> {
    let start = start_after.as_ref().map(|terra_acct| Bound::exclusive(terra_acct.as_str()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    CLAIMED
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (terra_acct, amount) = res?;
            Ok(ClaimedResponse {
                terra_acct,
                amount,
            })
        })
        .collect()
}

pub fn query_verify_signature(
    deps: Deps,
    terra_acct_pk: String,
    mars_acct: String,
    amount: Uint128,
    signature: String,
) -> StdResult<bool> {
    let terra_acct = pubkey_to_addr(&terra_acct_pk, "terra")?;

    verify_signature(deps.api, &msg(&terra_acct, &mars_acct, amount), &terra_acct_pk, &signature)
}

pub fn query_verify_proof(
    deps: Deps,
    terra_acct: String,
    amount: Uint128,
    merkle_proof: Vec<String>,
) -> StdResult<bool> {
    let merkle_root = ROOT.load(deps.storage)?;

    verify_proof(&leaf(&terra_acct, amount), &merkle_root, &merkle_proof)
}
