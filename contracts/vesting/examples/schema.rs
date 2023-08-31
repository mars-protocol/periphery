use cosmwasm_schema::write_api;
use mars_vesting::msg::{Config, ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: Config<String>,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
