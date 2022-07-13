/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  owner: string | undefined,
  quorum: number | undefined,
  threshold: number | undefined,
  voting_period: number | undefined,
  timelock_period: number | undefined,
  proposal_deposit: string | undefined,
  snapshot_period: number | undefined,
  unbonding_period: number | undefined,
  gov_hodler: string | undefined
): Promise<void> {
  process.stdout.write("Test - Only owner can update gov config");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      update_config: {
        owner,
        quorum,
        threshold,
        voting_period,
        timelock_period,
        proposal_deposit,
        snapshot_period,
        unbonding_period,
        gov_hodler,
      },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Update the configs of the Gov Hodler Contract
//----------------------------------------------------------------------------------------
export async function testGovHodlerUpdateConfig(
  juno: SigningCosmWasmClient,
  apTeam: string,
  gov_hodler: string,
  owner: string | undefined,
  govContract: string | undefined
): Promise<void> {
  process.stdout.write("Test - Only owner can update gov hodler config");

  await expect(
    sendTransaction(juno, apTeam, gov_hodler, {
      update_config: {
        owner,
        gov_contract: govContract,
      },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Transfer Stake from Old to New Gov Contract
//----------------------------------------------------------------------------------------
export async function testTransferStake(
  juno: SigningCosmWasmClient,
  apTeam: string,
  oldGov: string,
  newGov: string,
  staker_info: string[][]
): Promise<void> {
  process.stdout.write("Test - Execute transfer of staker balances to a new contract");
  let msgs: Msg[] = [];
  staker_info.forEach((info) => {
    msgs.push(
      new MsgExecuteContract(apTeam, oldGov, {
        transfer_stake: {
          new_gov_contract: newGov,
          address: info[0],
          amount: info[1],
        },
      })
    );
  });

  await expect(sendTransaction(juno, apTeam, msgs));
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a msgs of poll");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      end_poll: { poll_id },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      execute_poll: { poll_id },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      snapshot_poll: { poll_id },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  halo_token: string
): Promise<void> {
  process.stdout.write("Test - Gov register staking token contract");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      register_contracts: { halo_token },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloToken: string,
  govContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Stake voting tokens");

  await expect(
    sendTransaction(juno, apTeam, haloToken, {
      send: {
        amount: amount,
        contract: govContract,
        msg: toEncodedBinary({ stake_voting_tokens: {} }),
      },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  amount: string | undefined
): Promise<void> {
  process.stdout.write("Test - Withdraw voting tokens");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      withdraw_voting_tokens: { amount },
    })
  );
  console.log(chalk.green(" Passed!"));
}

export async function testGovClaimVotingTokens(
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Claim all eligable withdrawn voting tokens");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      claim_voting_tokens: {},
    })
  );
  console.log(chalk.green(" Passed!"));
}

export async function testGovResetClaims(
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  addresses: string[]
): Promise<void> {
  process.stdout.write("Test - Reset claims for all addresses passed");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      reset_claims: { claim_addrs: addresses },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  poll_id: number,
  vote: VoteOption,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Cast vote");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      cast_vote: { poll_id, vote, amount },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  halo_token: string,
  funding_goal: string | undefined,
  fund_rotation: number | undefined,
  split_to_liquid: string | undefined,
  treasury_tax_rate: string | undefined
): Promise<void> {
  process.stdout.write("Test - Execute a poll");

  await expect(
    sendTransaction(juno, apTeam, govContract, {
      receive: {
        sender: apTeam,
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
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryGovConfig(
  juno: SigningCosmWasmClient,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Query Gov Config");
  const result: any = await juno.queryContractSmart(govContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovState(
  juno: SigningCosmWasmClient,
  govContract: string
): Promise<void> {
  process.stdout.write("Test - Query Gov State");
  const result: any = await juno.queryContractSmart(govContract, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovStaker(
  juno: SigningCosmWasmClient,
  govContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query for getting gov staker");
  const result: any = await juno.queryContractSmart(govContract, {
    staker: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovClaims(
  juno: SigningCosmWasmClient,
  govContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Gov Claims for an addr");
  const result: any = await juno.queryContractSmart(govContract, {
    claims: { address },
  });

  // console.log(result);
  result.claims.forEach((r: any) => console.log(r, r.release_at));
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovPoll(
  juno: SigningCosmWasmClient,
  govContract: string,
  poll_id: number
): Promise<void> {
  process.stdout.write("Test - Query for getting poll by poll_id");
  const result: any = await juno.queryContractSmart(govContract, {
    poll: { poll_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovPolls(
  juno: SigningCosmWasmClient,
  govContract: string,
  filter: any | undefined,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query for getting poll by poll_id");
  const result: any = await juno.queryContractSmart(govContract, {
    polls: { filter, start_after, limit, undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGovVoters(
  juno: SigningCosmWasmClient,
  govContract: string,
  poll_id: number,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query gov voters");
  const result: any = await juno.queryContractSmart(govContract, {
    voters: { poll_id, start_after, limit, undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
