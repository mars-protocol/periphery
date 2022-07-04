#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Order, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::crypto::{pubkey_to_addr, verify_proof, verify_signature};
use crate::msg::{leaf, msg, ClaimedResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CLAIMED, DEADLINE, ROOT};
use crate::types::MarsMsg;

const CONTRACT_NAME: &str = "crates.io:mars-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ROOT.save(deps.storage, &msg.merkle_root)?;
    DEADLINE.save(deps.storage, &(env.block.time.seconds() + msg.claim_period))?;

    Ok(Response::new())
}

//--------------------------------------------------------------------------------------------------
// Executions
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<MarsMsg>> {
    let api = deps.api;
    match msg {
        ExecuteMsg::Claim {
            terra_acct_pk,
            mars_acct,
            amount,
            proof,
            signature,
        } => claim(
            deps,
            terra_acct_pk,
            api.addr_validate(&mars_acct)?,
            amount,
            proof,
            signature,
        ),
        ExecuteMsg::Clawback {} => clawback(deps, env),
    }
}

pub fn claim(
    deps: DepsMut,
    terra_acct_pk: String,
    mars_acct: Addr,
    amount: Uint128,
    proof: Vec<String>,
    signature: String,
) -> StdResult<Response<MarsMsg>> {
    let root = ROOT.load(deps.storage)?;

    let terra_acct = pubkey_to_addr(&terra_acct_pk, "terra")?;

    // The Terra account must not have already claimed
    CLAIMED.update(
        deps.storage,
        &terra_acct,
        |claimed| {
            if claimed.is_some() {
                return Err(StdError::generic_err("account has already claimed"));
            }
            Ok(amount)
        },
    )?;

    // The signature must be valid
    if !verify_signature(
        deps.api,
        &msg(&terra_acct, &mars_acct.to_string(), amount),
        &terra_acct_pk,
        &signature,
    )? {
        return Err(StdError::generic_err("invalid signature"));
    }

    // The Merkle proof must be valid
    if !verify_proof(
        &leaf(&terra_acct, amount),
        &root,
        &proof,
    )? {
        return Err(StdError::generic_err("invalid proof"));
    }

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: mars_acct.to_string(),
        amount: coins(amount.u128(), "umars"),
    });

    let event = Event::new("mars/airdrop/claimed")
        .add_attribute("terra_acct", terra_acct)
        .add_attribute("mars_acct", mars_acct)
        .add_attribute("amount", amount);

    Ok(Response::new().add_message(msg).add_event(event))
}

pub fn clawback(deps: DepsMut, env: Env) -> StdResult<Response<MarsMsg>> {
    let current_time = env.block.time.seconds();
    let deadline = DEADLINE.load(deps.storage)?;

    if current_time < deadline {
        return Err(StdError::generic_err(
            format!("clawback can only be invoked after {}", deadline),
        ));
    }

    let amount = deps.querier.query_all_balances(&env.contract.address)?;

    let amount_str = amount
        .iter()
        .map(|coin| coin.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let msg = CosmosMsg::Custom(MarsMsg::FundCommunityPool { amount });

    let event = Event::new("mars/airdrop/clawed_back")
        .add_attribute("timestamp", current_time.to_string())
        .add_attribute("amount", amount_str);

    Ok(Response::new().add_message(msg).add_event(event))
}

//--------------------------------------------------------------------------------------------------
// Queries
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Claimed {
            terra_acct,
        } => to_binary(&query_claimed(deps, terra_acct)?),
        QueryMsg::AllClaimed {
            start_after,
            limit,
        } => to_binary(&query_all_claimed(deps, start_after, limit)?),
        QueryMsg::VerifySignature {
            terra_acct_pk,
            mars_acct,
            amount,
            signature,
        } => to_binary(&query_verify_signature(deps, terra_acct_pk, mars_acct, amount, signature)?),
        QueryMsg::VerifyProof {
            terra_acct,
            amount,
            proof,
        } => to_binary(&query_verify_proof(deps, terra_acct, amount, proof)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        merkle_root: ROOT.load(deps.storage)?,
        claim_deadline: DEADLINE.load(deps.storage)?,
    })
}

pub fn query_claimed(deps: Deps, terra_acct: String) -> StdResult<ClaimedResponse> {
    Ok(ClaimedResponse {
        amount: CLAIMED.load(deps.storage, &terra_acct).unwrap_or_else(|_| Uint128::zero()),
        terra_acct,
    })
}

pub fn query_all_claimed(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<ClaimedResponse>>{
    let start = start_after.as_ref().map(|terra_acct| Bound::exclusive(terra_acct.as_str()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    CLAIMED
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| {
            let (terra_acct, amount) = res?;
            Ok(ClaimedResponse { terra_acct, amount })
        })
        .collect()
}

pub fn query_verify_signature(
    deps: Deps,
    terra_acct_pk: String,
    mars_acct: String,
    amount: Uint128,
    signature: String,
) -> StdResult<bool> {
    let terra_acct = pubkey_to_addr(&terra_acct_pk, "terra")?;

    verify_signature(
        deps.api,
        &msg(&terra_acct, &mars_acct, amount),
        &terra_acct_pk,
        &signature,
    )
}

pub fn query_verify_proof(
    deps: Deps,
    terra_acct: String,
    amount: Uint128,
    merkle_proof: Vec<String>,
) -> StdResult<bool> {
    let merkle_root = ROOT.load(deps.storage)?;

    verify_proof(
        &leaf(&terra_acct, amount),
        &merkle_root,
        &merkle_proof,
    )
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
        MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{from_binary, Empty, OwnedDeps, SubMsg, Timestamp};

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
            mock_env_at_timestamp(10000),
            mock_info("deployer", &[]),
            InstantiateMsg {
                merkle_root: "a7da979c32f9ffeca6214558c560780cf06b09e52fe670f16c532b20016d7f38".to_string(),
                claim_period: 10000,
            },
        )
        .unwrap();

        deps.querier.update_balance(MOCK_CONTRACT_ADDR, coins(1000000000, "umars"));

        deps
    }

    #[test]
    fn claiming() {
        let mut deps = setup_test();

        // valid test case generated by `scripts/1_build_merkle_tree.ts` and `scripts/2_sign_messages.ts`
        let terra_acct_pk = "02306e8b60d390b54aa36a79b825dfebc49b1f3483a110c448a36db2bdfebed248";
        let terra_acct = "terra1757tkx08n0cqrw7p86ny9lnxsqeth0wgp0em95";
        let mars_acct = "mars1757tkx08n0cqrw7p86ny9lnxsqeth0wg6k6zj0";
        let amount = Uint128::new(42069);
        let proof = vec![
            "43e1c4776372ff2136f9f8db4f2c9e8392ebd9c378bf47eeba18871309c453d7".to_string(),
            "37c71107165d3dc28551f006263453fa642d78fc013c04d2d89d96de022fde24".to_string(),
            "7089ea1db485169381b9e3539e5c61e3818b53c03a34f8f2aeecf35a3e112c3a".to_string(),
        ];
        let signature = "a0927f2beea637682263e91afd39c2e11f987e41c3239cc6e6a6d8bb9f07decc27c69c02968da59567449d2baf8c24990ddf0a6457fb1e7c6187e1cc6723483e";

        // valid proof, valid signature
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("claimer", &[]),
            ExecuteMsg::Claim {
                terra_acct_pk: terra_acct_pk.to_string(),
                mars_acct: mars_acct.to_string(),
                amount,
                proof: proof.clone(),
                signature: signature.to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: mars_acct.to_string(),
                amount: coins(42069, "umars"),
            })),
        );

        // "claimed" should have been updated
        let claimed = CLAIMED.load(deps.as_ref().storage, terra_acct).unwrap();
        assert_eq!(claimed, Uint128::new(42069));

        // the same account cannot claim twice
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("claimer", &[]),
            ExecuteMsg::Claim {
                terra_acct_pk: terra_acct_pk.to_string(),
                mars_acct: mars_acct.to_string(),
                amount,
                proof: proof.clone(),
                signature: signature.to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, StdError::generic_err("account has already claimed"));

        // reset "claimed" for the next test
        CLAIMED.remove(deps.as_mut().storage, terra_acct);

        // invalid proof, valid signature
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("claimer", &[]),
            ExecuteMsg::Claim {
                terra_acct_pk: terra_acct_pk.to_string(),
                mars_acct: mars_acct.to_string(),
                amount,
                proof: vec![
                    "f3712e76d8b9288a381de1d22720fe3673d9e2636f1c11b2b26d6e7889a78692".to_string(),
                    "37c71107165d3dc28551f006263453fa642d78fc013c04d2d89d96de022fde24".to_string(),
                    "bff2934478464bb50326325e6b2a2d47ba13475eccfa991e9825442b06ae7efc".to_string(),
                ],
                signature: signature.to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, StdError::generic_err("invalid proof"));

        // reset "claimed" for the next test
        CLAIMED.remove(deps.as_mut().storage, terra_acct);

        // valid proof, but invalid signature
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("claimer", &[]),
            ExecuteMsg::Claim {
                terra_acct_pk: terra_acct_pk.to_string(),
                mars_acct: mars_acct.to_string(),
                amount,
                proof: proof.clone(),
                signature: "7f73595b39e4e8ed853e3fbe49ca32438e9a9b6f1f578dfa6acfda1d267c60953b749ba2b091b6058c0258db3f9231a4529651962b718b77db3e0ed8887e7cd1".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, StdError::generic_err("invalid signature"));
    }

    #[test]
    fn querying_all_claimed() {
        let mut deps = setup_test();

        CLAIMED.save(deps.as_mut().storage, "larry", &Uint128::new(42069)).unwrap();
        CLAIMED.save(deps.as_mut().storage, "jake", &Uint128::new(69420)).unwrap();

        let res: Vec<ClaimedResponse> = query_helper(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AllClaimed {
                start_after: None,
                limit: None,
            },
        );
        assert_eq!(
            res,
            vec![
                ClaimedResponse {
                    terra_acct: "jake".to_string(),
                    amount: Uint128::new(69420),
                },
                ClaimedResponse {
                    terra_acct: "larry".to_string(),
                    amount: Uint128::new(42069),
                }
            ],
        );
    }

    #[test]
    fn clawing_back() {
        let mut deps = setup_test();

        // cannot claw back before the deadline
        let err = execute(
            deps.as_mut(),
            mock_env_at_timestamp(15000),
            mock_info("admin", &[]),
            ExecuteMsg::Clawback {},
        )
        .unwrap_err();

        assert_eq!(err, StdError::generic_err("clawback can only be invoked after 20000"));

        // can clawback at or after deadline
        let res = execute(
            deps.as_mut(),
            mock_env_at_timestamp(20000),
            mock_info("admin", &[]),
            ExecuteMsg::Clawback {},
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Custom(MarsMsg::FundCommunityPool {
                amount: coins(1000000000, "umars"),
            })),
        );
    }
}
