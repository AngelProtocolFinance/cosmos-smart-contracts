/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  endowmentContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransaction(juno, apTeam, endowmentContract, {
        deposit: {
          locked_percentage: "1",
          liquid_percentage: "0",
        },
      },
      { ujuno:  amount }
    )
  ).to.be.rejected;
  console.log(chalk.green(" Passed!"));
}

export async function testSingleDonationAmountToManyEndowments(
  juno: SigningCosmWasmClient,
  apTeam: string,
  endowments: string[],
  amount: string
): Promise<void> {
  process.stdout.write("Test - Send single amount to many Endowment Accounts");
  const msgs: Msg[] = endowments.map((endowment) => {
    return new MsgExecuteContract(
      apTeam,
      endowment,
      {
        deposit: {
          locked_percentage: "1",
          liquid_percentage: "0",
        },
      },
      { ujuno:  amount }
    );
  });
  await sendTransaction(juno, apTeam, msgs);
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
  juno: SigningCosmWasmClient,
  charityOwner: string,
  endowment: string,
  vault: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner cannot withdraw from the Endowment locked amount"
  );
  await expect(
    sendTransaction(juno, charityOwner, endowment, {
      withdraw: {
        sources: [{ vault, locked: "500000", liquid: "1000000" }],
        beneficiary,
        asset_info: {
          native: "ujuno"
        }
      }
    })
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Passed!"));

  process.stdout.write(
    "Test - Charity Owner can withdraw from the Endowment availalble amount (liquid)"
  );
  await expect(
    sendTransaction(juno, charityOwner, endowment, {
      withdraw: {
        sources: [{ vault, locked: "0", liquid: "30000000" }],
        beneficiary,
      },
    })
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
  juno: SigningCosmWasmClient,
  charity1: string,
  endowment: string,
  Vault1: string,
  Vault2: string
): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  await expect(
    sendTransaction(juno, charity1, endowment, {
      update_strategies: {
        strategies: [
          { vault: Vault1, percentage: "0.5"},
          { vault: Vault2, percentage: "0.5"},
        ],
      },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  endowment: string,
  owner: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");

  await expect(
    sendTransaction(juno, apTeam, endowment, {
      update_endowment_settings: {
        owner,
        beneficiary,
      },
    })
  );
  console.log(chalk.green(" Passed!"));
}

export async function testChangeManyAccountsEndowmentOwners(
  juno: SigningCosmWasmClient,
  apTeam: string,
  endowments: any[] // [ { address: <string>, owner: <string>, kyc_donors_only: <bool> }, ... ]
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");
  let msgs: Msg[] = [];
  endowments.forEach((e) => {
    msgs.push(
      new MsgExecuteContract(apTeam, e.address, {
        update_endowment_settings: {
          owner: e.owner,
          kyc_donors_only: e.kyc_donors_only,
        },
      })
    );
  });
  await expect(sendTransaction(juno, apTeam, msgs));
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryAccountsState(
  juno: SigningCosmWasmClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts State");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsTransactions(
  juno: SigningCosmWasmClient,
  endowmentContract: string,
  sender: string | undefined,
  recipient: string | undefined,
  denom: string | undefined
): Promise<void> {
  process.stdout.write("Test - Query Accounts Transactions");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
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
  juno: SigningCosmWasmClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Balance");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
    balance: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(
  juno: SigningCosmWasmClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowment(
  juno: SigningCosmWasmClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
    endowment: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsProfile(
  juno: SigningCosmWasmClient,
  endowmentContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Profile");
  const result: any = await juno.wasm.contractQuery(endowmentContract, {
    get_profile: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
