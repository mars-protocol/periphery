use cosmwasm_schema::write_api;
use mars_delegator::msg::ExecuteMsg;
use mars_delegator::msg::InstantiateMsg;
use mars_delegator::msg::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}