/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

export enum VoteOption {
  YES,
  NO
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
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update gov config");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        govContract,
        {
          update_config: {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period
          },
        },
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only owner can update gov config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        govContract,
        {
          update_config: {
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period
          },
        },
      ),
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
  poll_id: number,
): Promise<void> {
  process.stdout.write("Test - Execute a msgs of poll");

  await expect(
    sendTransaction(terra, apTeam, [ // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(
        govContract,
        govContract,
        {
          end_poll: { poll_id },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Execute a poll
//
// SCENARIO:
// Execute a msgs of passed poll as one submsg to catch failures
//
//----------------------------------------------------------------------------------------
export async function testGovExecutePoll(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  poll_id: number,
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(terra, apTeam, [ // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(
        govContract,
        govContract,
        {
          execute_poll: { poll_id },
        },
      ),
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
  poll_id: number,
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(terra, apTeam, [ // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(
        govContract,
        govContract,
        {
          snapshot_poll: { poll_id },
        },
      ),
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
      new MsgExecuteContract(
        apTeam.key.accAddress,
        govContract,
        {
          register_contracts: { halo_token },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Withdraw voting tokens
//
// SCENARIO:
// Withdraw amount if not staked. By default all funds will be withdrawn
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
      new MsgExecuteContract(
        apTeam.key.accAddress,
        govContract,
        {
          withdraw_voting_tokens: { amount },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Cast vote
//
// SCENARIO:
// Cast vote
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
      new MsgExecuteContract(
        apTeam.key.accAddress,
        govContract,
        {
          cast_vote: { poll_id, vote, amount },
        },
      ),
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