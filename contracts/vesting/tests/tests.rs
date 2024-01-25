use cosmwasm_std::{
    attr, coin, coins, from_binary,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Addr, BankMsg, CosmosMsg, Deps, Empty, Env, OwnedDeps, SubMsg, Timestamp, Uint128,
};
use cw2::{set_contract_version, ContractVersion, VersionError};
use cw_utils::PaymentError;
use mars_vesting::{
    contract::{execute, instantiate, migrate, query},
    error::Error,
    msg::{
        Config, ExecuteMsg, MigrateMsg, Position, PositionAlteration, PositionResponse, QueryMsg,
        Schedule, V1_1_1Updates, VotingPowerResponse,
    },
    state::POSITIONS,
};

pub const MOCK_DENOM: &str = "umars";

fn mock_unlock_schedule() -> Schedule {
    Schedule {
        start_time: 1662033600, // 2022-09-01
        cliff: 0,
        duration: 63072000, // two years (365 * 24 * 60 * 60 * 2)
    }
}

fn mock_env_at_timestamp(seconds: u64) -> Env {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(seconds);
    env
}

fn query_helper<T: serde::de::DeserializeOwned>(deps: Deps, env: Env, msg: QueryMsg) -> T {
    from_binary(&query(deps, env, msg).unwrap()).unwrap()
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();

    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("deployer", &[]),
        Config {
            owner: "owner".to_string(),
            denom: MOCK_DENOM.into(),
            unlock_schedule: mock_unlock_schedule(),
        },
    )
    .unwrap();

    deps
}

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let config: Config<String> = query_helper(deps.as_ref(), mock_env(), QueryMsg::Config {});
    assert_eq!(
        config,
        Config {
            owner: "owner".to_string(),
            denom: MOCK_DENOM.into(),
            unlock_schedule: mock_unlock_schedule(),
        },
    );
}

#[test]
fn updating_ownership() {
    let mut deps = setup_test();

    let new_cfg = Config {
        owner: "new_owner".into(),
        denom: MOCK_DENOM.into(),
        unlock_schedule: mock_unlock_schedule(),
    };

    // non-owner cannot transfer ownership
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("non_owner", &[]),
        ExecuteMsg::UpdateConfig {
            new_cfg: new_cfg.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(err, Error::NotOwner);

    // owner can propose a transfer
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            new_cfg,
        },
    )
    .unwrap();
    assert_eq!(res.messages.len(), 0);

    let config: Config<String> = query_helper(deps.as_ref(), mock_env(), QueryMsg::Config {});
    assert_eq!(config.owner, "new_owner".to_string());
}

#[test]
fn creating_positions() {
    let mut deps = setup_test();

    let msg = ExecuteMsg::CreatePosition {
        user: "larry".to_string(),
        vest_schedule: Schedule {
            start_time: 1614600000, // 2021-03-01
            cliff: 31536000,        // 1 year
            duration: 94608000,     // 3 years
        },
    };

    // non-owner cannot create positions
    let err =
        execute(deps.as_mut(), mock_env(), mock_info("non_owner", &[]), msg.clone()).unwrap_err();
    assert_eq!(err, Error::NotOwner);

    // cannot create a position without sending a coin
    let err = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg.clone()).unwrap_err();
    assert_eq!(err, PaymentError::NoFunds {}.into());

    // cannot create a position sending more than one coin
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(12345, "umars"), coin(23456, "uosmo")]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, PaymentError::MultipleDenoms {}.into());

    // cannot create a position with the wrong coin
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(23456, "uosmo")]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(err, PaymentError::MissingDenom(MOCK_DENOM.into()).into());

    // properly create a position
    let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[coin(12345, "umars")]), msg)
        .unwrap();
    assert_eq!(res.messages.len(), 0);

    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(
        position,
        Position {
            total: Uint128::new(12345),
            withdrawn: Uint128::zero(),
            vest_schedule: Schedule {
                start_time: 1614600000,
                cliff: 31536000,
                duration: 94608000,
            }
        },
    );
}

#[test]
fn terminating_positions() {
    let mut deps = setup_test();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(12345, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "larry".to_string(),
            vest_schedule: Schedule {
                start_time: 1614600000, // 2021-03-01
                cliff: 31536000,        // 1 year
                duration: 126144000,    // 4 years
            },
        },
    )
    .unwrap();

    // for this test, we simulate the most general case
    // the user first makes a withdrawal
    // 2022-10-01
    // vested:       12345 * (1664625600 - 1614600000) / 126144000 = 4895
    // unlocked:     12345 * (1664625600 - 1662033600) / 63072000  = 507
    // withdrawable: min(4895, 507) - 0 = 507
    execute(
        deps.as_mut(),
        mock_env_at_timestamp(1664625600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();

    // 2023-10-01
    // vested:       12345 * (1696161600 - 1614600000) / 126144000 = 7981
    // unlocked:     12345 * (1696161600 - 1662033600) / 63072000  = 6679
    // withdrawable: min(7981, 6679) - 507 = 6172
    let env = mock_env_at_timestamp(1696161600);

    let msg = ExecuteMsg::TerminatePosition {
        user: "larry".to_string(),
    };

    // non-owner can't terminate allocation
    let err =
        execute(deps.as_mut(), env.clone(), mock_info("non_owner", &[]), msg.clone()).unwrap_err();
    assert_eq!(err, Error::NotOwner);

    // owner properly terminates position
    let res = execute(deps.as_mut(), env, mock_info("owner", &[]), msg).unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(BankMsg::Send {
            to_address: "owner".to_string(),
            amount: coins(4364, "umars"), // 12345 - 7981
        })
    );

    // the position should have been updated
    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(
        position,
        Position {
            total: Uint128::new(7981),
            withdrawn: Uint128::new(507),
            vest_schedule: Schedule {
                start_time: 1614600000,
                cliff: 31536000,
                duration: 81561600
            }
        },
    );

    // voting power should be correct
    // total - withdrawn = 7981 - 507 = 7474
    let vpr: VotingPowerResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::VotingPower {
            user: "larry".to_string(),
        },
    );
    assert_eq!(vpr.voting_power, Uint128::new(7474));
}

#[test]
fn withdrawing() {
    let mut deps = setup_test();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(12345, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "larry".to_string(),
            vest_schedule: Schedule {
                start_time: 1614600000, // 2021-03-01
                cliff: 31536000,        // 1 year
                duration: 126144000,    // 4 years
            },
        },
    )
    .unwrap();

    // 2021-09-01
    // before the end of cliff period, withdrawable amount is zero
    let err = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1630497600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap_err();
    assert_eq!(err, Error::ZeroWithdrawable);

    // 2022-05-01
    // after the cliff period, but unlock hasn't start yet, withdrawable amount is zero
    let err = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1651406400),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap_err();
    assert_eq!(err, Error::ZeroWithdrawable);

    // 2022-10-01
    // vested:       12345 * (1664625600 - 1614600000) / 126144000 = 4895
    // unlocked:     12345 * (1664625600 - 1662033600) / 63072000  = 507
    // withdrawable: min(4895, 507) - 0 = 507
    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1664625600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "larry".to_string(),
            amount: coins(507, "umars"),
        })),
    );

    // withdrawn amount should have been recorded
    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(position.withdrawn, Uint128::new(507));

    // try immediately withdraw again in the same block, withdrawable amount should be zero
    let err = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1664625600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap_err();
    assert_eq!(err, Error::ZeroWithdrawable);

    // 2023-10-01
    // vested:       12345 * (1696161600 - 1614600000) / 126144000 = 7981
    // unlocked:     12345 * (1696161600 - 1662033600) / 63072000  = 6679
    // withdrawable: min(7981, 6679) - 507 = 6172
    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1696161600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "larry".to_string(),
            amount: coins(6172, "umars"),
        })),
    );

    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(position.withdrawn, Uint128::new(6679));

    // 2024-10-01
    // vested:       12345 * (1727784000 - 1614600000) / 126144000 = 11076
    // unlocked:     12345 (unlocking finished)
    // withdrawable: min(11076, 12345) - 6679 = 4397
    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1727784000),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "larry".to_string(),
            amount: coins(4397, "umars"),
        })),
    );

    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(position.withdrawn, Uint128::new(11076));

    // 2025-10-01
    // withdrawable: 12345 - 11076 = 1269
    let res = execute(
        deps.as_mut(),
        mock_env_at_timestamp(1759320000),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "larry".to_string(),
            amount: coins(1269, "umars"),
        })),
    );

    let position = POSITIONS.load(deps.as_ref().storage, &Addr::unchecked("larry")).unwrap();
    assert_eq!(position.withdrawn, Uint128::new(12345));
}

#[test]
fn querying_positions() {
    let mut deps = setup_test();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(12345, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "larry".to_string(),
            vest_schedule: Schedule {
                start_time: 1614600000, // 2021-03-01
                cliff: 31536000,        // 1 year
                duration: 126144000,    // 4 years
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(23456, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "jake".to_string(),
            vest_schedule: Schedule {
                start_time: 612964800, // 1989-06-04
                cliff: 0,
                duration: 1040688000, // 33 years
            },
        },
    )
    .unwrap();

    // larry withdraws some - 507 umars
    execute(
        deps.as_mut(),
        mock_env_at_timestamp(1664625600),
        mock_info("larry", &[]),
        ExecuteMsg::Withdraw {},
    )
    .unwrap();

    // 2023-10-01
    //
    // larry
    // vested:       12345 * (1696161600 - 1614600000) / 126144000 = 7981
    // unlocked:     12345 * (1696161600 - 1662033600) / 63072000  = 6679
    // withdrawable: min(7981, 6679) - 507 = 6172
    //
    // jake
    // vested:       23456 (vesting finished)
    // unlocked:     23456 * (1696161600 - 1662033600) / 63072000 = 12691
    // withdrawable: min(23456, 12691) - 0 = 12691
    let expected_larry = PositionResponse {
        user: "larry".to_string(),
        total: Uint128::new(12345),
        vested: Uint128::new(7981),
        unlocked: Uint128::new(6679),
        withdrawn: Uint128::new(507),
        withdrawable: Uint128::new(6172),
        vest_schedule: Schedule {
            start_time: 1614600000, // 2021-03-01
            cliff: 31536000,        // 1 year
            duration: 126144000,    // 4 years
        },
    };
    let expected_jake = PositionResponse {
        user: "jake".to_string(),
        total: Uint128::new(23456),
        vested: Uint128::new(23456),
        unlocked: Uint128::new(12691),
        withdrawn: Uint128::zero(),
        withdrawable: Uint128::new(12691),
        vest_schedule: Schedule {
            start_time: 612964800, // 1989-06-04
            cliff: 0,
            duration: 1040688000, // 33 years
        },
    };

    let res: PositionResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::Position {
            user: "larry".to_string(),
        },
    );
    assert_eq!(res, expected_larry);

    let res: PositionResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::Position {
            user: "jake".to_string(),
        },
    );
    assert_eq!(res, expected_jake);

    let res: Vec<PositionResponse> = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::Positions {
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], expected_jake);
    assert_eq!(res[1], expected_larry);

    let res: Vec<PositionResponse> = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::Positions {
            start_after: None,
            limit: Some(1),
        },
    );
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], expected_jake);

    let res: Vec<PositionResponse> = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::Positions {
            start_after: Some("jake".to_string()),
            limit: None,
        },
    );
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], expected_larry);

    // voting power
    // larry: 12345 - 507         = 11838
    // jake:  23456 - 0           = 23456
    // total: 12345 + 23456 - 507 = 35294
    let vpr: VotingPowerResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::VotingPower {
            user: "larry".to_string(),
        },
    );
    assert_eq!(vpr.voting_power, Uint128::new(11838));

    let vpr: VotingPowerResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::VotingPower {
            user: "jake".to_string(),
        },
    );
    assert_eq!(vpr.voting_power, Uint128::new(23456));

    // query the voting power of a user who doesn't have a vesting position; should return zero
    let vpr: VotingPowerResponse = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::VotingPower {
            user: "pumpkin".to_string(),
        },
    );
    assert_eq!(vpr.voting_power, Uint128::zero());

    let vprs: Vec<VotingPowerResponse> = query_helper(
        deps.as_ref(),
        mock_env_at_timestamp(1696161600),
        QueryMsg::VotingPowers {
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(
        vprs,
        vec![
            VotingPowerResponse {
                user: "jake".to_string(),
                voting_power: Uint128::new(23456),
            },
            VotingPowerResponse {
                user: "larry".to_string(),
                voting_power: Uint128::new(11838),
            }
        ],
    );
}

#[test]
fn invalid_contract_version() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let old_contract_version = ContractVersion {
        contract: "crates.io:mars-vesting".to_string(),
        version: "1.0.0".to_string(),
    };

    set_contract_version(
        deps.as_mut().storage,
        old_contract_version.contract.clone(),
        old_contract_version.version,
    )
    .unwrap();

    let update_msg = V1_1_1Updates {
        position_alterations: vec![],
        total_reclaim: Uint128::new(123),
    };

    let err = migrate(deps.as_mut(), env, MigrateMsg::V1_1_0ToV1_1_1(update_msg)).unwrap_err();
    assert_eq!(
        Error::Version(VersionError::WrongVersion {
            expected: "1.1.0".to_string(),
            found: "1.0.0".to_string()
        }),
        err
    );
}

#[test]
fn proper_migration() {
    let mut deps = setup_test();
    cw2::set_contract_version(deps.as_mut().storage, "crates.io:mars-vesting", "1.1.0").unwrap();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(456, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "larry".to_string(),
            vest_schedule: Schedule {
                start_time: 1614600000, // 2021-03-01
                cliff: 31536000,        // 1 year
                duration: 126144000,    // 4 years
            },
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[coin(789, "umars")]),
        ExecuteMsg::CreatePosition {
            user: "piotr".to_string(),
            vest_schedule: Schedule {
                start_time: 1614600000, // 2021-03-01
                cliff: 31536000,        // 1 year
                duration: 126144000,    // 4 years
            },
        },
    )
    .unwrap();

    let update_msg = V1_1_1Updates {
        position_alterations: vec![
            PositionAlteration {
                addr: Addr::unchecked("larry"),
                total_old: Uint128::new(456),
                total_new: Uint128::new(333),
                reclaim: Uint128::new(123),
            },
            PositionAlteration {
                addr: Addr::unchecked("piotr"),
                total_old: Uint128::new(789),
                total_new: Uint128::new(666),
                reclaim: Uint128::new(123),
            },
        ],
        total_reclaim: Uint128::new(246),
    };

    let res = migrate(deps.as_mut(), mock_env(), MigrateMsg::V1_1_0ToV1_1_1(update_msg)).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "owner".to_string(),
            amount: coins(246, "umars")
        }))]
    );
    assert!(res.data.is_none());
    assert_eq!(
        res.attributes,
        vec![attr("action", "migrate"), attr("from_version", "1.1.0"), attr("to_version", "1.1.1"),]
    );
}
