use cosmwasm_std::{Deps, StdResult};

use crate::state::ENDING_TIME;

pub fn query_ending_time(deps: Deps) -> StdResult<u64> {
    ENDING_TIME.load(deps.storage)
}
