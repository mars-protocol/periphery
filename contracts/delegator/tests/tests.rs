use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    coin, coins, Addr, Decimal, Empty, Env, FullDelegation, OwnedDeps, StakingMsg, SubMsg,
    Timestamp, Validator,
};

use mars_delegator::contract::{execute, instantiate, sudo};
use mars_delegator::msg::{Config, ExecuteMsg, InstantiateMsg, SudoMsg};
use mars_delegator::state::CONFIG;
use mars_types::MarsMsg;

pub const BOND_DENOM: &str = "umars";

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

    // give the contract some coin balance. this is for testing refunding
    deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(10000, BOND_DENOM));

    // instantiate the contract
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("deployer", &coins(10000, BOND_DENOM)),
        InstantiateMsg {
            bond_denom: BOND_DENOM.into(),
            ending_time: 10000,
        },
    )
    .unwrap();

    deps
}

#[test]
fn instantiating() {
    let mut deps = setup_test();

    let res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("deployer", &coins(10000, BOND_DENOM)),
        InstantiateMsg {
            bond_denom: BOND_DENOM.into(),
            ending_time: 10000,
        },
    )
    .unwrap();

    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(StakingMsg::Delegate {
                validator: "larry".into(),
                amount: coin(3334, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Delegate {
                validator: "jake".into(),
                amount: coin(3333, BOND_DENOM)
            }),
            SubMsg::new(StakingMsg::Delegate {
                validator: "pumpkin".into(),
                amount: coin(3333, BOND_DENOM)
            }),
        ]
    );

    let cfg = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        cfg,
        Config {
            bond_denom: "umars".into(),
            ending_time: 10000,
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

    // cannot unbond before the ending time is reached
    {
        let _err = execute(
            deps.as_mut(),
            mock_env_at_timestamp(9999),
            mock_info("larry", &[]),
            ExecuteMsg::Unbond {},
        )
        .unwrap_err();
        // I would like to assert the correct error, but Rust won't let me:
        // binary operation `==` cannot be applied to type `ContractError`
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

    {
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

        let _err =
            execute(deps.as_mut(), mock_env(), mock_info("larry", &[]), ExecuteMsg::Refund {})
                .unwrap_err();
        // I would like to assert the correct error, but Rust won't let me:
        // binary operation `==` cannot be applied to type `ContractError`
    }
}
