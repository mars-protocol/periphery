use std::collections::BTreeSet;

use cosmwasm_std::{coin, Addr, DepsMut, Env, QuerierWrapper, Response, StakingMsg, StdResult};

use mars_types::MarsMsg;

use crate::error::ContractError;
use crate::state::{BOND_DENOM, ENDING_TIME};

pub fn init(deps: DepsMut, bond_denom: String) -> Result<Response, ContractError> {
    BOND_DENOM.save(deps.storage, &bond_denom)?;

    Ok(Response::new())
}

pub fn bond(
    deps: DepsMut,
    env: Env,
    validators: BTreeSet<String>,
    ending_time: u64,
) -> Result<Response<MarsMsg>, ContractError> {
    let bond_denom = BOND_DENOM.load(deps.storage)?;

    let balance = deps.querier.query_balance(&env.contract.address, &bond_denom)?;
    if balance.amount.is_zero() {
        return Err(ContractError::NothingToBond);
    }

    let current_time = env.block.time.seconds();
    if ending_time <= current_time {
        return Err(ContractError::InvalidEndingTime {
            ending_time,
            current_time,
        });
    }

    ENDING_TIME.save(deps.storage, &ending_time)?;

    Ok(Response::new()
        .add_messages(get_delegation_msgs(
            &deps.querier,
            validators,
            balance.amount.u128(),
            &bond_denom,
        )?)
        .add_attribute("action", "periphery/delegator/bond")
        .add_attribute("amount", balance.to_string()))
}

pub fn force_unbond(deps: DepsMut, env: Env) -> Result<Response<MarsMsg>, ContractError> {
    Ok(Response::new()
        .add_messages(get_undelegate_msgs(&deps.querier, &env.contract.address)?)
        .add_attribute("action", "periphery/delegator/force_unbond"))
}

pub fn unbond(deps: DepsMut, env: Env) -> Result<Response<MarsMsg>, ContractError> {
    let ending_time = ENDING_TIME.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    if current_time < ending_time {
        return Err(ContractError::EndingTimeNotReached {
            ending_time,
            current_time,
        });
    }

    Ok(Response::new()
        .add_messages(get_undelegate_msgs(&deps.querier, &env.contract.address)?)
        .add_attribute("action", "periphery/delegator/unbond"))
}

pub fn refund(deps: DepsMut, env: Env) -> Result<Response<MarsMsg>, ContractError> {
    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    if amount.is_empty() {
        return Err(ContractError::NothingToRefund);
    }

    Ok(Response::new()
        .add_message(MarsMsg::FundCommunityPool {
            amount,
        })
        .add_attribute("action", "periphery/delegator/refund"))
}

/// Generate messages to delegate evenly to each validator in the specified list.
///
/// Need to handle the case where the coin balance is not divisible by the
/// number of validators.
/// For this we use the same algorithm from Steak:
/// https://github.com/steak-enjoyers/steak/blob/v2.0.0-rc0/contracts/hub/src/math.rs#L52-L90
///
/// NOTE: We don't handle the case where the number of validators is zero,
/// because it's impossible.
pub fn get_delegation_msgs(
    querier: &QuerierWrapper,
    validator_addrs: BTreeSet<String>,
    amount: u128,
    denom: &str,
) -> Result<Vec<StakingMsg>, ContractError> {
    let validators = validator_addrs
        .into_iter()
        .map(|address| {
            querier.query_validator(&address)?.ok_or_else(|| ContractError::ValidatorNotFound {
                address,
            })
        })
        .collect::<Result<Vec<_>, ContractError>>()?;

    let num_validators = validators.len() as u128;
    let tokens_per_validator = amount / num_validators;
    let remainder = amount % num_validators;

    Ok(validators
        .into_iter()
        .enumerate()
        .map(|(idx, validator)| {
            let remainder_for_validator = if (idx + 1) as u128 <= remainder {
                1
            } else {
                0
            };
            let tokens_for_validator = tokens_per_validator + remainder_for_validator;
            StakingMsg::Delegate {
                validator: validator.address,
                amount: coin(tokens_for_validator, denom),
            }
        })
        .collect())
}

/// Query current delegations made to validators, and generate messages to undelegate from them.
pub fn get_undelegate_msgs(
    querier: &QuerierWrapper,
    delegator_addr: &Addr,
) -> StdResult<Vec<StakingMsg>> {
    Ok(querier
        .query_all_delegations(delegator_addr)?
        .into_iter()
        .map(|delegation| StakingMsg::Undelegate {
            validator: delegation.validator,
            amount: delegation.amount,
        })
        .collect())
}
