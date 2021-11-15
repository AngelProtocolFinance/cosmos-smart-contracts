/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

type VestingAccount = {
  address: string,
  schedules: [number, number, string][]
}

//----------------------------------------------------------------------------------------
// TEST: Update vesting config
//
// SCENARIO:
// Owner can update config
//
//----------------------------------------------------------------------------------------
export async function testVestingUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  vestingContract: string,
  owner: string | undefined,
  halo_token: string | undefined,
  genesis_time: number | undefined
): Promise<void> {
  process.stdout.write("Test - Owner can update vesting config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        vestingContract,
        {
          update_config: {
            owner,
            halo_token,
            genesis_time
          },
        },
      ),
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
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  vestingContract: string,
  vesting_accounts: VestingAccount[],
): Promise<void> {
  process.stdout.write("Test - Register vesting account");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        vestingContract,
        {
          register_vesting_accounts: { vesting_accounts },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Update vesting account
//
// SCENARIO:
// Resiger vesting accounts
//
//----------------------------------------------------------------------------------------
export async function testVestingUpdateVestingAccount(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  vestingContract: string,
  vesting_account: VestingAccount,
): Promise<void> {
  process.stdout.write("Test - Register vesting account");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        vestingContract,
        {
          update_vesting_account: { vesting_account },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryVestingConfig(
  terra: LocalTerra | LCDClient,
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
  terra: LocalTerra | LCDClient,
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
  terra: LocalTerra | LCDClient,
  vestingContract: string,
  start_after: string | undefined,
  limit: number | undefined
): Promise<void> {
  process.stdout.write("Test - Query vesting accounts");
  const result: any = await terra.wasm.contractQuery(vestingContract, {
    vesting_accounts: {
      start_after,
      limit,
      order_by: undefined
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
