use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coin, to_binary, BankMsg, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};

use crate::helpers::{setup_test, setup_test_with_balance};
use mars_liquidation_filterer::contract::execute;
use mars_liquidation_filterer::error::ContractError;
use mars_liquidation_filterer::msg::ExecuteMsg;
use mars_liquidation_filterer::types::Liquidate;
use mars_outpost::red_bank::{UserHealthStatus, UserPositionResponse};

mod helpers;

// We are only interested in health status, the rest can have random values
fn dummy_user_position(health_status: UserHealthStatus) -> UserPositionResponse {
    UserPositionResponse {
        total_enabled_collateral: Default::default(),
        total_collateralized_debt: Default::default(),
        weighted_max_ltv_collateral: Default::default(),
        weighted_liquidation_threshold_collateral: Default::default(),
        health_status,
    }
}

#[test]
fn test_liquidate_many_accounts_if_missing_debt_coin() {
    let mut deps = setup_test();
    deps.querier.set_redbank_user_position(
        "user_address_1".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );
    deps.querier.set_redbank_user_position(
        "user_address_2".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );

    let info = mock_info("owner", &[coin(1234u128, "uosmo")]);
    let msg = ExecuteMsg::LiquidateMany {
        liquidations: vec![
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "uosmo".to_string(),
                user_address: "user_address_1".to_string(),
                amount: Uint128::from(10u32),
            },
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "umars".to_string(),
                user_address: "user_address_2".to_string(),
                amount: Uint128::from(10u32),
            },
        ],
    };
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidFunds {
            reason: "missing umars".to_string()
        }
    );
}

#[test]
fn test_liquidate_many_accounts_if_not_enough_debt_coin() {
    let mut deps = setup_test();
    deps.querier.set_redbank_user_position(
        "user_address_1".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );
    deps.querier.set_redbank_user_position(
        "user_address_2".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );

    let info = mock_info("owner", &[coin(20u128, "umars"), coin(10u128, "uosmo")]);
    let msg = ExecuteMsg::LiquidateMany {
        liquidations: vec![
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "uosmo".to_string(),
                user_address: "user_address_1".to_string(),
                amount: Uint128::from(10u32),
            },
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "umars".to_string(),
                user_address: "user_address_2".to_string(),
                amount: Uint128::from(10u32),
            },
            Liquidate {
                collateral_denom: "ujuno".to_string(),
                debt_denom: "umars".to_string(),
                user_address: "user_address_2".to_string(),
                amount: Uint128::from(11u32),
            },
        ],
    };
    let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidFunds {
            reason: "not enough umars".to_string()
        }
    );
}

#[test]
fn test_liquidate_many_accounts() {
    let mut deps = setup_test();
    deps.querier.set_redbank_user_position(
        "user_address_1".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );
    deps.querier.set_redbank_user_position(
        "user_address_2".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(110),
            liq_threshold_hf: Decimal::percent(120),
        }),
    );
    deps.querier.set_redbank_user_position(
        "user_address_3".to_string(),
        dummy_user_position(UserHealthStatus::Borrowing {
            max_ltv_hf: Decimal::percent(80),
            liq_threshold_hf: Decimal::percent(90),
        }),
    );
    deps.querier.set_redbank_user_position(
        "user_address_4".to_string(),
        dummy_user_position(UserHealthStatus::NotBorrowing),
    );

    let info = mock_info(
        "bot",
        &[
            coin(1234u128, "uosmo"),
            coin(2345u128, "umars"),
            coin(3456u128, "ujuno"),
            coin(7891u128, "uaxelar"),
        ],
    );
    let msg = ExecuteMsg::LiquidateMany {
        liquidations: vec![
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "umars".to_string(),
                user_address: "user_address_1".to_string(),
                amount: Uint128::from(345u32),
            },
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "uosmo".to_string(),
                user_address: "user_address_2".to_string(),
                amount: Uint128::from(234u32),
            },
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "uaxelar".to_string(),
                user_address: "user_address_3".to_string(),
                amount: Uint128::from(891u32),
            },
            Liquidate {
                collateral_denom: "uatom".to_string(),
                debt_denom: "ujuno".to_string(),
                user_address: "user_address_4".to_string(),
                amount: Uint128::from(456u32),
            },
        ],
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    // user_address_2 is healthy, user_address_4 is not borrowing, additional refund message
    assert_eq!(res.messages.len(), 3);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "red_bank".to_string(),
            msg: to_binary(&mars_outpost::red_bank::ExecuteMsg::Liquidate {
                collateral_denom: "uatom".to_string(),
                user: "user_address_1".to_string()
            })
            .unwrap(),
            funds: vec![coin(345u128, "umars")]
        }))
    );
    assert_eq!(
        res.messages[1],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "red_bank".to_string(),
            msg: to_binary(&mars_outpost::red_bank::ExecuteMsg::Liquidate {
                collateral_denom: "uatom".to_string(),
                user: "user_address_3".to_string()
            })
            .unwrap(),
            funds: vec![coin(891u128, "uaxelar")]
        }))
    );
    assert_eq!(
        res.messages[2],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "cosmos2contract".to_string(),
            msg: to_binary(&ExecuteMsg::Refund {
                recipient: "bot".to_string()
            })
            .unwrap(),
            funds: vec![]
        }))
    );
}

#[test]
fn test_refund_if_no_coins() {
    let mut deps = setup_test();

    let info = mock_info("contract", &[]);
    let msg = ExecuteMsg::Refund {
        recipient: "bot".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_refund() {
    let mut deps = setup_test_with_balance(&[coin(1234u128, "uosmo"), coin(2345u128, "umars")]);

    let info = mock_info("contract", &[]);
    let msg = ExecuteMsg::Refund {
        recipient: "bot".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "bot".to_string(),
            amount: vec![coin(1234u128, "uosmo"), coin(2345u128, "umars")],
        }))
    );
}
