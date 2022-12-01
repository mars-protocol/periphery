use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::ConfigResponse,
    state::{BOND_DENOM, ENDING_TIME},
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        bond_denom: BOND_DENOM.load(deps.storage)?,
        ending_time: ENDING_TIME.may_load(deps.storage)?,
    })
}
