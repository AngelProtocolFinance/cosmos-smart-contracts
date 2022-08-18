/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendMessageViaCw3Proposal, sendTransaction, sendTransactionWithFunds } from "../../../utils/helpers";

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
    sendTransactionWithFunds(juno, apTeam, endowmentContract, {
        deposit: {
          locked_percentage: "1",
          liquid_percentage: "0",
        },
      },
      [{ denom: "ujuno", amount: amount }]
    )
  ).to.be.rejected;
  console.log(chalk.green(" Passed!"));
}

export async function testSendDonationToEndowment(
  juno: SigningCosmWasmClient,
  apTeam: string,
  accounts: string,
  endowId: number,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Send single amount to an Endowment Account");
  await expect(
    sendTransactionWithFunds(juno, apTeam, accounts, {
        deposit: {
          id: endowId,
          locked_percentage: "1",
          liquid_percentage: "0",
        },
      },
      [{ denom: "ujuno", amount }]
    )
  ).to.be.ok;
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
  cw3: string,
  endowment: string,
  vault: string,
  amount: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner cannot withdraw from the Endowment amount"
  );
  await expect(
    sendMessageViaCw3Proposal(juno, charityOwner, cw3, endowment, {
      withdraw: {
        sources: [{ vault, amount }],
        beneficiary,
        asset_info: {
          native: "ujuno"
        }
      },
    })
  ).to.be.ok;
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
  accounts: string,
  endowId: number,
  Vault1: string,
  Vault2: string
): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  const res = await juno.queryContractSmart(accounts, { endowment: { id: endowId }});
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, charity1, cw3, accounts, {
      update_strategies: {
        id: endowId,
        strategies: [
          { vault: Vault1, percentage: "0.5"},
          { vault: Vault2, percentage: "0.5"},
        ],
      },
    })
  ).to.be.ok;
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
  beneficiary: string,
  kyc_donors_only: boolean,
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");

  await expect(
    sendTransaction(juno, apTeam, endowment, {
      update_endowment_settings: {
        owner,
        beneficiary,
        kyc_donors_only,
      },
    })
  );
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
  const result = await juno.queryContractSmart(endowmentContract, {
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
  const result = await juno.queryContractSmart(endowmentContract, {
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
  const result = await juno.queryContractSmart(endowmentContract, {
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
  const result = await juno.queryContractSmart(endowmentContract, {
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
  const result = await juno.queryContractSmart(endowmentContract, {
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
  const result = await juno.queryContractSmart(endowmentContract, {
    get_profile: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
