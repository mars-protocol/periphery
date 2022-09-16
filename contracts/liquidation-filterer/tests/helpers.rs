use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Coin, OwnedDeps};

use mars_liquidation_filterer::contract::instantiate;
use mars_liquidation_filterer::msg::InstantiateMsg;
use mars_testing::{mock_dependencies, MarsMockQuerier};

pub fn setup_test() -> OwnedDeps<MockStorage, MockApi, MarsMockQuerier> {
    setup_test_with_balance(&[])
}

pub fn setup_test_with_balance(
    balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MarsMockQuerier> {
    let mut deps = mock_dependencies(balance);

    let msg = InstantiateMsg {
        owner: String::from("owner"),
        address_provider: String::from("address_provider"),
    };
    let info = mock_info("owner", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    deps
}
