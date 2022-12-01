use std::collections::BTreeSet;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    coin, coins, from_binary, Addr, Decimal, Empty, Env, FullDelegation, OwnedDeps, StakingMsg,
    SubMsg, Timestamp, Validator,
};

use mars_delegator::contract::{execute, instantiate, query, sudo};
use mars_delegator::error::ContractError;
use mars_delegator::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use mars_delegator::state::ENDING_TIME;
use mars_types::MarsMsg;

pub const BOND_DENOM: &str = "umars";

/// Collect a &[&str] into BTreeSet<String>
fn btreeset(validators: &[&str]) -> BTreeSet<String> {
    validators.to_vec().into_iter().map(String::from).collect()
}

fn mock_env_at_timestamp(timestamp: u64) -> Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(timestamp);
    env
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    // initialize a mock validator set with three validators
    deps.querier.update_staking(
        "umars",
        &[
            Validator {
                address: "larry".into(),
                commission: Decimal::zero(),
                max_commission: Decimal::zero(),
                max_change_rate: Decimal::zero(),
            },
            Validator {
                address: "jake".into(),
                commission: Decimal::zero(),
                max_commission: Decimal::zero(),
                max_change_rate: Decimal::zero(),
            },
            Validator {
                address: "pumpkin".into(),
                commission: Decimal::zero(),
                max_commission: Decimal::zero(),
                max_change_rate: Decimal::zero(),
            },
        ],
        &[
            FullDelegation {
                delegator: Addr::unchecked(MOCK_CONTRACT_ADDR),
                validator: "larry".into(),
                amount: coin(3334, BOND_DENOM),
                can_redelegate: coin(3334, BOND_DENOM),
                accumulated_rewards: vec![],
            },
            FullDelegation {
                delegator: Addr::unchecked(MOCK_CONTRACT_ADDR),
                validator: "jake".into(),
                amount: coin(3333, BOND_DENOM),
                can_redelegate: coin(3333, BOND_DENOM),
                accumulated_rewards: vec![],
            },
            FullDelegation {
                delegator: Addr::unchecked(MOCK_CONTRACT_ADDR),
                validator: "pumpkin".into(),
                amount: coin(3333, BOND_DENOM),
                can_redelegate: coin(3333, BOND_DENOM),
                accumulated_rewards: vec![],
            },
        ],
    );

    // instantiate the contract
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("deployer", &coins(10000, BOND_DENOM)),
        InstantiateMsg {
            bond_denom: BOND_DENOM.into(),
        },
    )
    .unwrap();

    deps
}

#[test]
fn instantiating() {
    let deps = setup_test();

    // bond denom should have been saved
    // ending time should have not been specified yet
    let cfg_bytes = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let cfg: ConfigResponse = from_binary(&cfg_bytes).unwrap();
    assert_eq!(
        cfg,
        ConfigResponse {
            bond_denom: "umars".into(),
            ending_time: None,
        },
    );
}

#[test]
fn bonding() {
    let mut deps = setup_test();

    // prior to invoking `bond`, governance must have passed a community pool
    // spending proposal to transfer some tokens to the contract.
    // here we give the contract 10000 umars
    deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(10000, BOND_DENOM));

    let res = sudo(
        deps.as_mut(),
        mock_env_at_timestamp(1),
        SudoMsg::Bond {
            validators: btreeset(&["larry", "pumpkin", "jake"]),
            ending_time: 10000,
        },
    )
    .unwrap();

    // NOTE: delegate messages are sorted alphabetically by validator addresses
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(StakingMsg::Delegate {
                validator: "jake".into(),
                amount: coin(3334, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Delegate {
                validator: "larry".into(),
                amount: coin(3333, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Delegate {
                validator: "pumpkin".into(),
                amount: coin(3333, BOND_DENOM)
            })
        ],
    );

    // the ending time should have been updated
    let cfg_bytes = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let cfg: ConfigResponse = from_binary(&cfg_bytes).unwrap();
    assert_eq!(
        cfg,
        ConfigResponse {
            bond_denom: "umars".into(),
            ending_time: Some(10000),
        },
    );
}

#[test]
fn forced_unbonding() {
    let mut deps = setup_test();

    let res = sudo(deps.as_mut(), mock_env(), SudoMsg::ForceUnbond {}).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(StakingMsg::Undelegate {
                validator: "larry".into(),
                amount: coin(3334, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Undelegate {
                validator: "jake".into(),
                amount: coin(3333, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Undelegate {
                validator: "pumpkin".into(),
                amount: coin(3333, BOND_DENOM)
            })
        ]
    );
}

#[test]
fn unbonding() {
    let mut deps = setup_test();

    // ending time is set by the `bond` sudo method.
    // here we just set it
    ENDING_TIME.save(deps.as_mut().storage, &10000).unwrap();

    // cannot unbond before the ending time is reached
    {
        let err = execute(
            deps.as_mut(),
            mock_env_at_timestamp(9999),
            mock_info("larry", &[]),
            ExecuteMsg::Unbond {},
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::EndingTimeNotReached {
                ending_time: 10000,
                current_time: 9999
            }
        );
    }

    // can unbond after ending time is reached
    {
        let res = execute(
            deps.as_mut(),
            mock_env_at_timestamp(69420),
            mock_info("larry", &[]),
            ExecuteMsg::Unbond {},
        )
        .unwrap();
        assert_eq!(
            res.messages,
            vec![
                SubMsg::new(StakingMsg::Undelegate {
                    validator: "larry".into(),
                    amount: coin(3334, BOND_DENOM)
                }),
                SubMsg::new(StakingMsg::Undelegate {
                    validator: "jake".into(),
                    amount: coin(3333, BOND_DENOM)
                }),
                SubMsg::new(StakingMsg::Undelegate {
                    validator: "pumpkin".into(),
                    amount: coin(3333, BOND_DENOM)
                })
            ]
        );
    }
}

#[test]
fn refunding() {
    let mut deps = setup_test();

    // give the contract some coin balance to be refunded
    deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(10000, BOND_DENOM));

    {
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(10000, BOND_DENOM));
        let res =
            execute(deps.as_mut(), mock_env(), mock_info("larry", &[]), ExecuteMsg::Refund {})
                .unwrap();
        assert_eq!(
            res.messages,
            vec![SubMsg::new(MarsMsg::FundCommunityPool {
                amount: coins(10000, BOND_DENOM)
            })]
        );
    }

    // test the case where the contract has no coin balance
    {
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, vec![]);

        let err =
            execute(deps.as_mut(), mock_env(), mock_info("larry", &[]), ExecuteMsg::Refund {})
                .unwrap_err();
        assert_eq!(err, ContractError::NothingToRefund);
    }
}
