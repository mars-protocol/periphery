use cosmwasm_std::{coins, BankMsg, CosmosMsg, DepsMut, Response, Uint128};
use cw2::set_contract_version;

use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::Result,
    msg::V1_1_1Updates,
    state::{CONFIG, POSITIONS},
};

const FROM_VERSION: &str = "1.1.0";

pub fn migrate(deps: DepsMut, msg: V1_1_1Updates) -> Result<Response> {
    // make sure we're migrating the correct contract and from the correct version
    cw2::assert_contract_version(deps.as_ref().storage, CONTRACT_NAME, FROM_VERSION)?;

    let mut total_reclaim: Uint128 = Uint128::new(0);

    for position_alteration in msg.position_alterations {
        let mut position = POSITIONS.load(deps.storage, &position_alteration.addr)?;

        // Determine amount to send back to owner
        let reclaim = position.total.checked_sub(position_alteration.total_new)?;
        total_reclaim = total_reclaim.checked_add(reclaim)?;

        // Confirm state is as expected
        assert!(position.total == position_alteration.total_old);
        assert!(position.withdrawn.is_zero());
        assert!(reclaim == position_alteration.reclaim);

        // Set new total vesting amount
        position.total = position_alteration.total_new;
        POSITIONS.save(deps.storage, &position_alteration.addr, &position)?;
    }

    // Additoinal check that the total amount reclaimed back is as expected
    assert!(total_reclaim == msg.total_reclaim);

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = CONFIG.load(deps.storage)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: cfg.owner.to_string(),
            amount: coins(total_reclaim.u128(), cfg.denom),
        }))
        .add_attribute("action", "migrate")
        .add_attribute("from_version", FROM_VERSION)
        .add_attribute("to_version", CONTRACT_VERSION))
}
