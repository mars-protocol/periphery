use cosmwasm_schema::write_api;
use mars_airdrop::msg::QueryMsg;
use mars_airdrop::msg::InstantiateMsg;
use mars_airdrop::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}