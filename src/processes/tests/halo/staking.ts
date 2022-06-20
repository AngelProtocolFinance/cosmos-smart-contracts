/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LcdClient,  MsgExecuteContract, Wallet } from "@cosmjs/launchpad";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Unbond
//
// SCENARIO:
// Cannot unbond more than bond amount
//
//----------------------------------------------------------------------------------------
export async function testStakingUnbond(
  juno: LcdClient,
  apTeam: Wallet,
  stakingContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Unbond less than bond amount");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        stakingContract,
        {
          unbond: { amount },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Withdraw
//
// SCENARIO:
// Withdraw rewards to executor
//
//----------------------------------------------------------------------------------------
export async function testStakingWithdraw(
  juno: LcdClient,
  apTeam: Wallet,
  stakingContract: string
): Promise<void> {
  process.stdout.write("Test - Withdraw rewards to executor");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        stakingContract,
        {
          withdraw: {},
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryStakingConfig(
  juno: LcdClient,
  stakingContract: string
): Promise<void> {
  process.stdout.write("Test - Query Staking Config");
  const result: any = await terra.wasm.contractQuery(stakingContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryStakingState(
  juno: LcdClient,
  stakingContract: string
): Promise<void> {
  process.stdout.write("Test - Query Staking State");
  const result: any = await terra.wasm.contractQuery(stakingContract, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryStakingStakerInfo(
  juno: LcdClient,
  stakingContract: string,
  staker: string,
  block_height: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query Airdrop Latest Stage");
  const result: any = await terra.wasm.contractQuery(stakingContract, {
    staker_info: { staker, block_height },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
