/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LcdClient,  MsgExecuteContract, Wallet } from "@cosmjs/launchpad";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

type VestingAccount = {
  address: string;
  schedules: [number, number, string][];
};

//----------------------------------------------------------------------------------------
// TEST: Update vesting config
//
// SCENARIO:
// Owner can update config
//
//----------------------------------------------------------------------------------------
export async function testVestingUpdateConfig(
  juno: LcdClient,
  apTeam: Wallet,
  vestingContract: string,
  owner: string | undefined,
  halo_token: string | undefined,
  genesis_time: number | undefined
): Promise<void> {
  process.stdout.write("Test - Owner can update vesting config");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, vestingContract, {
        update_config: {
          owner,
          halo_token,
          genesis_time,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Register vesting accounts
//
// SCENARIO:
// Resiger vesting accounts
//
//----------------------------------------------------------------------------------------
export async function testVestingRegisterVestingAccounts(
  juno: LcdClient,
  apTeam: Wallet,
  vestingContract: string,
  vesting_accounts: VestingAccount[]
): Promise<void> {
  process.stdout.write("Test - Register vesting account");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, vestingContract, {
        register_vesting_accounts: { vesting_accounts },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Add new  vesting account
//
// SCENARIO:
// Add some number of new schedules to an existing vesting account
//
//----------------------------------------------------------------------------------------
export async function testAddSchedulesToVestingAccount(
  juno: LcdClient,
  apTeam: Wallet,
  address: string,
  newSchedules: [number, number, string][]
): Promise<void> {
  process.stdout.write("Test - Add new schedules to existing vesting account");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, vestingContract, {
        add_schedules_to_vesting_account: {
          address,
          new_schedules: newSchedules,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: User claims vested tokens from vesting contract
//
// SCENARIO:
// User has tokens that are available for claiming from the vesting contract
//
//----------------------------------------------------------------------------------------
export async function testUserClaimsVestedTokens(
  juno: LcdClient,
  apTeam: Wallet,
  vestingContract: string
): Promise<void> {
  process.stdout.write("Test - User can claim available tokens from the vesting account");

  await expect(
    sendTransaction(juno, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, vestingContract, {
        claim: {},
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryVestingConfig(
  juno: LcdClient,
  vestingContract: string
): Promise<void> {
  process.stdout.write("Test - Query Vesting Config");
  const result: any = await terra.wasm.contractQuery(vestingContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVestingAccount(
  juno: LcdClient,
  vestingContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query get vesting account by address");
  const result: any = await terra.wasm.contractQuery(vestingContract, {
    vesting_account: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVestingAccounts(
  juno: LcdClient,
  vestingContract: string,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query vesting accounts");
  const result: any = await terra.wasm.contractQuery(vestingContract, {
    vesting_accounts: {
      start_after,
      limit,
      order_by: undefined,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
