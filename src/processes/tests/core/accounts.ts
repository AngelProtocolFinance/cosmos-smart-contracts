/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {
  LCDClient,
  LocalTerra,
  Msg,
  MsgExecuteContract,
  Wallet,
} from "@terra-money/terra.js";
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
  apTeam: Wallet,
  endowmentContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        endowmentContract,
        {
          deposit: {
            locked_percentage: "1",
            liquid_percentage: "0",
          },
        },
        { uluna:  amount }
      ),
    ])
  // ); //.to.be.rejectedWith("Request failed with status code 400");
  ).to.be.rejected;
  console.log(chalk.green(" Passed!"));
}

export async function testSingleDonationAmountToManyEndowments(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  endowments: string[],
  amount: string
): Promise<void> {
  process.stdout.write("Test - Send single amount to many Endowment Accounts");
  const msgs: Msg[] = endowments.map((endowment) => {
    return new MsgExecuteContract(
      apTeam.key.accAddress,
      endowment,
      {
        deposit: {
          locked_percentage: "1",
          liquid_percentage: "0",
        },
      },
      { uluna:  amount }
    );
  });
  await sendTransaction(terra, apTeam, msgs);
  console.log(chalk.green(" Passed!"));
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
  charityOwner: Wallet,
  endowment: string,
  vault: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner cannot withdraw from the Endowment locked amount"
  );
  await expect(
    sendTransaction(terra, charityOwner, [
      new MsgExecuteContract(charityOwner.key.accAddress, endowment, {
        withdraw: {
          sources: [{ vault, locked: "500000", liquid: "1000000" }],
          beneficiary,
          asset_info: {
            native: "uluna"
          }
        },
      }),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Passed!"));

  process.stdout.write(
    "Test - Charity Owner can withdraw from the Endowment availalble amount (liquid)"
  );
  await expect(
    sendTransaction(terra, charityOwner, [
      new MsgExecuteContract(charityOwner.key.accAddress, endowment, {
        withdraw: {
          sources: [{ vault, locked: "0", liquid: "30000000" }],
          beneficiary,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
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
  endowment: string,
  anchorVault1: string,
  anchorVault2: string
): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowment, {
        update_strategies: {
          strategies: [
            { vault: anchorVault1, percentage: "0.5"},
            { vault: anchorVault2, percentage: "0.5"},
          ],
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Contract Owner can set new owner of endowment
//
// SCENARIO:
// Contract owner needs to change the endowment owner from single wallet to a CW3 multisig
//
//----------------------------------------------------------------------------------------

export async function testApTeamChangesAccountsEndowmentOwner(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  endowment: string,
  owner: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, endowment, {
        update_endowment_settings: {
          owner,
          beneficiary,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

export async function testChangeManyAccountsEndowmentOwners(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  endowments: any[] // [ { address: <string>, owner: <string>, kyc_donors_only: <bool> }, ... ]
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");
  let msgs: Msg[] = [];
  endowments.forEach((e) => {
    msgs.push(
      new MsgExecuteContract(apTeam.key.accAddress, e.address, {
        update_endowment_settings: {
          owner: e.owner,
          kyc_donors_only: e.kyc_donors_only,
        },
      })
    );
  });
  await expect(sendTransaction(terra, apTeam, msgs));
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryAccountsState(
  terra: LocalTerra | LCDClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts State");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsTransactions(
  terra: LocalTerra | LCDClient,
  endowmentContract: string,
  sender: string | undefined,
  recipient: string | undefined,
  denom: string | undefined
): Promise<void> {
  process.stdout.write("Test - Query Accounts Transactions");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    get_tx_records: {
      sender,
      recipient,
      denom,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsBalance(
  terra: LocalTerra | LCDClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Balance");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    balance: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(
  terra: LocalTerra | LCDClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowment(
  terra: LocalTerra | LCDClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    endowment: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsProfile(
  terra: LocalTerra | LCDClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Profile");
  const result: any = await terra.wasm.contractQuery(endowmentContract, {
    get_profile: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
