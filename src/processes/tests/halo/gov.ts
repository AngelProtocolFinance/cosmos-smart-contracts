/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

export enum VoteOption {
  YES,
  NO,
}

//----------------------------------------------------------------------------------------
// TEST: Update gov config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testGovUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pleb: Wallet,
  govContract: string,
  owner: string | undefined,
  quorum: number | undefined,
  threshold: number | undefined,
  voting_period: number | undefined,
  timelock_period: number | undefined,
  proposal_deposit: string | undefined,
  snapshot_period: number | undefined,
  unbonding_period: number | undefined
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update gov config");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(pleb.key.accAddress, govContract, {
        update_config: {
          owner,
          quorum,
          threshold,
          voting_period,
          timelock_period,
          proposal_deposit,
          snapshot_period,
          unbonding_period,
        },
      }),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Passed!"));

  process.stdout.write("Test - Only owner can update gov config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        update_config: {
          quorum,
          threshold,
          voting_period,
          timelock_period,
          proposal_deposit,
          snapshot_period,
          unbonding_period,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: End a poll
//
// SCENARIO:
// End a poll
//
//----------------------------------------------------------------------------------------
export async function testGovEndPoll(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a msgs of poll");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        end_poll: { poll_id },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Execute a poll
//
// SCENARIO:
// User can execute a poll that has passed vote
// User cannot execute a poll that has not passed or expired vote
//
//----------------------------------------------------------------------------------------
export async function testGovExecutePoll(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        execute_poll: { poll_id },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Snapshot a poll
//
// SCENARIO:
// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
//
//----------------------------------------------------------------------------------------
export async function testGovSnapshotPoll(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(terra, apTeam, [
      // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(govContract, govContract, {
        snapshot_poll: { poll_id },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Register contracts
//
// SCENARIO:
// Register contracts
//
//----------------------------------------------------------------------------------------
export async function testGovRegisterContracts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  halo_token: string
): Promise<void> {
  process.stdout.write("Test - Airdrop claim");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        register_contracts: { halo_token },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Stake HALO voting tokens
//
// SCENARIO:
// Stake some amount of HALO tokens for voting on Polls
//
//----------------------------------------------------------------------------------------
export async function testGovStakeVotingTokens(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  haloToken: string,
  govContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Stake voting tokens");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, haloToken, {
        send: {
          amount: amount,
          contract: govContract,
          msg: toEncodedBinary({ stake_voting_tokens: {} }),
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Withdraw voting tokens
//
// SCENARIO:
// Withdraw amount if not staked. By default all funds will be withdrawn.
// Withdrawn HALO is subject to an unbonding period before it can be accessed.
//
//----------------------------------------------------------------------------------------
export async function testGovWithdrawVotingTokens(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  amount: string | undefined
): Promise<void> {
  process.stdout.write("Test - Withdraw voting tokens");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        withdraw_voting_tokens: { amount },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

export async function testGovClaimVotingTokens(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Claim all eligable withdrawn voting tokens");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        claim_voting_tokens: {},
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Cast vote
//
// SCENARIO:
// AP Team/User can vote on the open poll
//
//----------------------------------------------------------------------------------------
export async function testGovCastVote(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  poll_id: number,
  vote: VoteOption,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Cast vote");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        cast_vote: { poll_id, vote, amount },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Execute a poll
//
// SCENARIO:
// Submit a poll for changing registrar settings
//
//----------------------------------------------------------------------------------------
export async function testGovExecutePollForRegistrarSettings(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  halo_token: string,
  funding_goal: string | undefined,
  fund_rotation: number | undefined,
  split_to_liquid: string | undefined,
  treasury_tax_rate: string | undefined
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(terra, apTeam, [
      // TODO: replace apTeam to HALO Token(Wallet)
      new MsgExecuteContract(apTeam.key.accAddress, govContract, {
        receive: {
          sender: apTeam.key.accAddress,
          amount: "123",
          msg: toEncodedBinary({
            title: "title",
            description: "description",
            link: undefined,
            proposal_type: "registrar",
            options: [
              {
                order: 1,
                funding_goal,
                fund_rotation,
                split_to_liquid,
                treasury_tax_rate,
                msg: toEncodedBinary({ amount: "123" }),
              },
            ],
          }),
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryGovConfig(
  terra: LocalTerra | LCDClient,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Query Gov Config");
  const result: any = await terra.wasm.contractQuery(govContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovState(
  terra: LocalTerra | LCDClient,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Query Gov State");
  const result: any = await terra.wasm.contractQuery(govContract, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovStaker(
  terra: LocalTerra | LCDClient,
  govContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query for getting gov staker");
  const result: any = await terra.wasm.contractQuery(govContract, {
    staker: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovClaims(
  terra: LocalTerra | LCDClient,
  govContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Gov Config");
  const result: any = await terra.wasm.contractQuery(govContract, {
    claims: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovPoll(
  terra: LocalTerra | LCDClient,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Query for getting poll by poll_id");
  const result: any = await terra.wasm.contractQuery(govContract, {
    poll: { poll_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovPolls(
  terra: LocalTerra | LCDClient,
  govContract: string,
  filter: any | undefined,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query for getting poll by poll_id");
  const result: any = await terra.wasm.contractQuery(govContract, {
    polls: { filter, start_after, limit, undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovVoters(
  terra: LocalTerra | LCDClient,
  govContract: string,
  poll_id: number,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query gov voters");
  const result: any = await terra.wasm.contractQuery(govContract, {
    voters: { poll_id, start_after, limit, undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
