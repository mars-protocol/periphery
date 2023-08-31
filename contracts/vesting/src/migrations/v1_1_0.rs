use cosmwasm_std::{DepsMut, Response};
use cw2::set_contract_version;

use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::Result,
    msg::Config,
    state::CONFIG,
};

const FROM_VERSION: &str = "1.0.0";

pub mod v1_0_0_state {
    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;

    use crate::msg::Schedule;

    pub const OWNER: Item<Addr> = Item::new("owner");
    pub const VEST_DENOM: &str = "umars";
    pub const UNLOCK_SCHEDULE: Item<Schedule> = Item::new("unlock_schedule");
}

pub fn migrate(deps: DepsMut) -> Result<Response> {
    // make sure we're migrating the correct contract and from the correct version
    cw2::assert_contract_version(deps.as_ref().storage, CONTRACT_NAME, FROM_VERSION)?;

    // CONFIG updated, re-initializing
    let cfg = Config {
        owner: v1_0_0_state::OWNER.load(deps.storage)?,
        denom: v1_0_0_state::VEST_DENOM.into(),
        unlock_schedule: v1_0_0_state::UNLOCK_SCHEDULE.load(deps.storage)?,
    };

    CONFIG.save(deps.storage, &cfg)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("from_version", FROM_VERSION)
        .add_attribute("to_version", CONTRACT_VERSION))
}
