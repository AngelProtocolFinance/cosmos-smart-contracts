# Fundraising

## Overview 
This is an fundraising contract that allows endowments to create Campaigns and for multiple users to contribute. The basic function is the sender (the "creator") creates a Campaign with some amount of locked funds to be used as rewards for users (the "contributors") who contribute to the Campaign. Each Campaign has a unique id (for future calls to reference it).

## Anatomy of a Campaign
Fundraising contract holds many `Campaign`s. Each Campaign has a few important characteristics: 
- Unique ID (`u64`) - Auto assigned at time of creation
- End time: When does the Campaign end?
- Funding Goal: What token does the endowment want to raise and how much of that token. Can only be 1 (Cw20 OR Native) __for now (with this POC)__.
- Funding Threshold: Cut off amount of tokens that the Campaign needs to raise in order to "succeed". 
- Contributed Balance: How much has been raised so far from users?

## Closing a Campaign
Once a Campaign passes the `end_time_epoch`, no new contributions can be accepted. At this point, the Campaign can now be "closed" by anyone. 
During closing, the Campaign's comtributed balance is checked against it's threshold.
- IF `Contributed >= Threshold`? **SUCCESS** - Payout
    1. All contributed funds are sent to the Campaign creator
    2. Fee/Tax owed to AP Treasury is sent (based on final amount raised/contributed)
    3. All Rewards owed to users can now be claimed (with `ClaimRewards` endpoint)
- IF `Contributed < Threshold`: **FAIL** - Reverse all funds flows
    1. All locked rewards funds are sent back to the Campaign creator
    2. All funds contributed by users can now be refunded to them (with `RefundContributions` endpoint)

## Additonal functions
We also add a function called `top_up`, which allows a Campaign creator to add more
funds to the contract at any time.

We added a function called `contribute` which allows anyone to contribute valid tokens to 
a Campaign while it is active.

Once a Campaign has reached the time limit then the Campaign will end.

Once ended: 
- If the `funding_threshold` is met then contributed funds will be release to the Campaign creator and reward funds are claimable by contributors.
- If the `funding_threshold` is NOT met then all contributed funds will be re-claimable by contributors and the rewards funds returned to the Campaign creator.
 

## Token types
This contract is meant not just to be functional, but also to work as a simple
example of an CW20 "Receiver". And demonstrate how the same calls can be fed
native tokens (via typical `ExecuteMsg` route), or cw20 tokens (via `Receiver` interface).

Both `create` and `top_up` can be called directly (with a payload of native tokens),
or from a CW20 contract using the [Receiver Interface](../../packages/cw20/README.md#receiver).
This means we can load a Campaign with any number of native or cw20 tokens 

```
FUTURE WORK IDEA: 
We could allow for Campaign's to support a mix of tokens with only a bit of refactoring to this POC contract!
```

## Running this contract
You will need Rust 1.44.1+ with `wasm32-unknown-unknown` target installed.
You can run unit tests on this via: 
`cargo test`

Once you are happy with the content, you can compile it to wasm via:
```
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw20_escrow.wasm .
ls -l cw20_escrow.wasm
sha256sum cw20_escrow.wasm
```

Or for a production-ready (optimized) build, run a build command in the
the repository root: https://github.com/CosmWasm/cw-plus#compiling.
