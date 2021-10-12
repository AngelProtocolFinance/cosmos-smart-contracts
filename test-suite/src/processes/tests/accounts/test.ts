/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Cannot send funds to an Endowment that is not approved for deposits 
//
// SCENARIO:
// If an Endowment has not been approved by the AP Team, all deposits should be rejected
//
//----------------------------------------------------------------------------------------

export async function testRejectUnapprovedDonations(
  terra: LocalTerra | LCDClient,
  pleb: Wallet,
  endowmentContract3: string
): Promise<void> {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        endowmentContract3,
        {
          deposit: {
            locked_percentage: "0.8",
            liquid_percentage: "0.2",
          },
        },
        { uusd: "4200000", }
      ),
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Charity Beneficiary can withdraw from available balance in their Accounts
//
// SCENARIO:
// Charity beneficiary can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------
export async function testBeneficiaryCanWithdrawFromLiquid(
  terra: LocalTerra | LCDClient,
  charity1: Wallet,
  endowmentContract1: string,
  anchorVault1: string,
  anchorVault2: string
): Promise<void> {
  process.stdout.write("Test - Beneficiary can withdraw from the Endowment availalble amount (liquid)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "500000", liquid: "1000000"},
            {vault: anchorVault2, locked: "500000", liquid: "1000000"}
          ]
        }
      })
    ])
  ).to.be.rejectedWith("Cannot withdraw from Locked balances");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "0", liquid: "30000"},
          ]
        }
      })
    ])
  );

  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Charity Owner can rebalance their portfolio/update the Accounts' strategy
//
// SCENARIO:
// Charity Owner can trigger a rebalance of their Accounts, which should: 
// 1) redeem all invested funds from Vaults to the Accounts
// 2) reinvest all redeemed funds, according the accounts' strategy
//
//----------------------------------------------------------------------------------------

export async function testCharityCanUpdateStrategies(
  terra: LocalTerra | LCDClient,
  charity1: Wallet,
  endowmentContract1: string,
  anchorVault1: string,
  anchorVault2: string
): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        update_strategies: {
          strategies: [
            {vault: anchorVault1, locked_percentage: "0.5", liquid_percentage: "0.5"},
            {vault: anchorVault2, locked_percentage: "0.5", liquid_percentage: "0.5"}
          ]
        }
      })
    ])
  );
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryAccountsBalance(
  terra: LocalTerra | LCDClient,
  endowmentContract1: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Balance");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    balance: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(
  terra: LocalTerra | LCDClient,
  endowmentContract1: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowment(
  terra: LocalTerra | LCDClient,
  endowmentContract1: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    endowment: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
