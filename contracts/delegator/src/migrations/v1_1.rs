use cosmwasm_std::{coin, DepsMut, Response, StakingMsg};

use crate::{
    contract::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
};

const EXPECTED_VERSION: &str = "1.0.0";

const BOND_DENOM: &str = "umars";

/// Injective validator address
const REDELEGATE_TO: &str = "marsvaloper1mvdau3mafeyf7xga5w76t636lwcyf5s0jx7zmk";

/// The other 15 validators, and how many tokens are to be redelegated to
/// Injective from each of them.
const REDELEGATE_FROM: [(&str, u128); 15] = [
    ("marsvaloper1yh9t45dtzws6tfllg95d423wa7t0hnss9fp2gl", 208333333334),
    ("marsvaloper1956ywsyr2uxhgy65425lkgu3fqr6lysmw26dl2", 208333333334),
    ("marsvaloper1xwazl8ftks4gn00y5x3c47auquc62ssuv6rkhw", 208333333334),
    ("marsvaloper1tzg9mnrp5mu0qk394pmk962fcj7nl63ykrwtd4", 208333333334),
    ("marsvaloper1trvtq89lkes7qjqhm5cyqltgvdu3jpej9ytn6e", 208333333334),
    ("marsvaloper1t7jn0208cmk5xky0vhvd0pzec3zuv9z5wwyrmd", 208333333333),
    ("marsvaloper1v8fkm5qj6lzguwvavj2ms62ekeday824w6c8cs", 208333333333),
    ("marsvaloper1vdjsxu4vzjlplkmfh5skt4g4r0y3lv52e9a4vs", 208333333333),
    ("marsvaloper1dj5ml59qu4vyxd790h2yt62dh7hapmk8hux0da", 208333333333),
    ("marsvaloper1weg2ynyy3qqe9nwanp2ga4ph8k8evdnsry3fjx", 208333333333),
    ("marsvaloper1w6rcmalz58suc6rv9mxrpygh4q60er85jf929z", 208333333333),
    ("marsvaloper1srlnshalj6mm092z45jwp8prpuqjnm9xgt8htw", 208333333333),
    ("marsvaloper1hvtaqw9mlwc0a4cdx6g3klk8acfc6z3yazzk8a", 208333333333),
    ("marsvaloper16pj5gljqnqs0ajxakccfjhu05yczp9870ral7t", 208333333333),
    ("marsvaloper17s96kckqgumhwl26hd3ek580m0fkk008an2nx0", 208333333333),
];

pub fn migrate(deps: DepsMut) -> Result<Response, ContractError> {
    let version = cw2::get_contract_version(deps.as_ref().storage)?;

    // can only migrate mars-delegator contract
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::incorrect_contract(CONTRACT_NAME, version.contract));
    }

    // can only migrate from v1.0.0
    if version.version != EXPECTED_VERSION {
        return Err(ContractError::incorrect_version(EXPECTED_VERSION, version.version));
    }

    // update contract version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // compose redelegate messages
    let msgs = REDELEGATE_FROM.iter().map(|(from, amount)| StakingMsg::Redelegate {
        src_validator: (*from).into(),
        dst_validator: REDELEGATE_TO.into(),
        amount: coin(*amount, BOND_DENOM),
    });

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "migrate")
        .add_attribute("contract", CONTRACT_NAME)
        .add_attribute("old_version", version.version)
        .add_attribute("new_version", CONTRACT_VERSION))
}
