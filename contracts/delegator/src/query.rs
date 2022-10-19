use cosmwasm_std::{Deps, StdResult};

use crate::{msg::Config, state::CONFIG};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}
