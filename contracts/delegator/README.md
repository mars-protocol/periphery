# Mars Delegator

The purpose of this contract is **to bootstrap the security of Mars Hub blockchain**.

At launch, Mars Hub will have a genesis validator set of 10–20 members, each having exactly 1 MARS token. The decision was made that each genesis validator only gets 1 token and not more, such that they don't have an unfair advantage over those who join post-genesis. However, this also means that for a brief period after launch, the network will have a very low security (10–20 MARS worth of security, to be specific). If a user is eligible for a big airdrop, they can potentially create a validator with the airdrop tokens right after launch and hijack the network.

To mitigate this risk, the Mars community pool will have a portion of its tokens (~1% of total supply may be a reasonable amount) deposited into this "delegator" contract, which then delegates the tokens evenly to each of the genesis validators. At instantiation, the contract will be given an `ending_time` for these delegations. Once the ending time is elapsed, anyone can invoke a method on the contract to unbond these delegations. Once unbonding is completed, anyone can invoke `refund` to return all funds to the community pool.

## License

Contents of this crate are open source under [GNU General Public License v3](../../LICENSE) or later.
