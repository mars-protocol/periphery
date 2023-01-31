use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use mars_delegator::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        migrate: Empty,
    }
}
