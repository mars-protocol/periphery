#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    coin, Addr, DepsMut, Env, MessageInfo, QuerierWrapper, Response, StakingMsg, StdResult,
};
use cw_utils::must_pay;

use mars_types::MarsMsg;

use crate::error::ContractError;
use crate::msg::Config;
use crate::state::CONFIG;

pub fn init(deps: DepsMut, info: MessageInfo, cfg: Config) -> Result<Response, ContractError> {
    // We don't implement a validity check of the ending time.
    // The deployer must make sure to provide a valid value.
    CONFIG.save(deps.storage, &cfg)?;

    let amount = must_pay(&info, &cfg.bond_denom)?;
    let msgs = get_delegation_msgs(&deps.querier, amount.u128(), &cfg.bond_denom)?;

    Ok(Response::new().add_messages(msgs))
}

pub fn force_unbond(deps: DepsMut, env: Env) -> StdResult<Response<MarsMsg>> {
    let msgs = get_undelegate_msgs(&deps.querier, &env.contract.address)?;
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "periphery/delegator/force_unbond"))
}

pub fn unbond(deps: DepsMut, env: Env) -> Result<Response<MarsMsg>, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    if current_time < cfg.ending_time {
        return Err(ContractError::ending_time_not_reached(cfg.ending_time, current_time));
    }

    let msgs = get_undelegate_msgs(&deps.querier, &env.contract.address)?;

    Ok(Response::new().add_messages(msgs).add_attribute("action", "periphery/delegator/unbond"))
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

/// Query the validator set, and generate messages to delegate evenly to each validator.
///
/// Need to handle the case where the coin balance is not divisible by the number of validators.
/// For this we use the same algorithm from Steak:
/// https://github.com/steak-enjoyers/steak/blob/v2.0.0-rc0/contracts/hub/src/math.rs#L52-L90
///
/// NOTE: We don't handle the case where the number of validators is zero, because it's impossible.
pub fn get_delegation_msgs(
    querier: &QuerierWrapper,
    amount: u128,
    denom: &str,
) -> StdResult<Vec<StakingMsg>> {
    let validators = querier.query_all_validators()?;
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
