use cosmwasm_schema::write_api;
use mars_vesting::msg::InstantiateMsg;
use mars_vesting::msg::ExecuteMsg;
use mars_vesting::msg::QueryMsg;




fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}