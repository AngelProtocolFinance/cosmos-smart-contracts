/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction, sendTransactionWithFunds, sendMessageViaCw3Proposal, sendApplicationViaCw3Proposal } from "../../../utils/helpers";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

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
  accountsContract: string,
  endowmentId: number,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransactionWithFunds(juno, apTeam, accountsContract, {
      deposit: {
        id: endowmentId,
        locked_percentage: "1",
        liquid_percentage: "0",
      },
    },
      [{ denom: "ujuno", amount: amount }]
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

export async function testSendDonationToEndowment(
  juno: SigningCosmWasmClient,
  apTeam: string,
  accountsContract: string,
  endowmentId: number,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Send single amount to an Endowment Account");
  await expect(
    sendTransactionWithFunds(juno, apTeam, accountsContract, {
      deposit: {
        id: endowmentId,
        locked_percentage: "0.5",
        liquid_percentage: "0.5",
      },
    },
      [{ denom: "ujuno", amount }]
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Endowment owner can withdraw from available balance in their Accounts
//
// SCENARIO:
// Endowment owner can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------
export async function testEndowmentCanWithdraw(
  juno: SigningCosmWasmClient,
  accountsOwner: string,
  accountsContract: string,
  endowmentId: number,
  acct_type: string,
  beneficiary: string,
  assets: any,
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner can withdraw from the Endowment amount"
  );

  const res = await juno.queryContractSmart(accountsContract, { config: {} });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, accountsOwner, cw3, accountsContract, {
      withdraw: {
        id: endowmentId,
        acct_type,
        beneficiary,
        assets,
      },
    })
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: Endowment owner can withdraw from available balance in their Accounts
//
// SCENARIO:
// Endowment owner can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------
export async function testEndowmentVaultsRedeem(
  juno: SigningCosmWasmClient,
  accountsOwner: string,
  accountsContract: string,
  endowmentId: number,
  acct_type: string,
  vaults: any,
): Promise<void> {
  process.stdout.write(
    "Test - Endowment can redeem the vault token"
  );

  const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, accountsOwner, cw3, accountsContract, {
      vaults_redeem: {
        id: endowmentId,
        acct_type: acct_type,
        vaults,
      },
    })
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
  accounts: string,
  endowId: number,
  vault: string,
  amount: string,
  beneficiary: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner cannot withdraw from the Endowment amount liquid"
  );

  const res = await juno.queryContractSmart(accounts, { endowment: { id: endowId } });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, charityOwner, cw3, accounts, {
      withdraw_liquid: {
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
  charity: string,
  accountsContract: string,
  endowmentId: number,
  acct_type: string,
  strategies: any, // [ { vault: string, percentage: "decimal" }, ... ]
): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, charity, cw3, accountsContract, {
      update_strategies: {
        id: endowmentId,
        acct_type,
        strategies,
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
  accountsContract: string,
  endowmentId: number,
  owner: string,
  beneficiary: string,
  kyc_donors_only: boolean,
): Promise<void> {
  process.stdout.write("Test - Contract Owner can set new owner of an Endowment");

  await expect(
    sendTransaction(juno, apTeam, accountsContract, {
      update_endowment_settings: {
        id: endowmentId,
        owner,
        beneficiary,
        kyc_donors_only,
      },
    })
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Endowment created from the Registrar
//
// SCENARIO:
// User sends request to create a new endowment to the Registrar
//
//----------------------------------------------------------------------------------------
export async function testCreateEndowment(
  networkUrl: string,
  proposerWallet: DirectSecp256k1HdWallet,
  cw3ReviewTeam: string,
  accounts: string,
  msg: any,
  members: DirectSecp256k1HdWallet[],  // Should be [apTeam]
): Promise<void> {
  process.stdout.write("Create a new endowment via the CW3 Applications contract");
  let endow_id = await sendApplicationViaCw3Proposal(networkUrl, proposerWallet, cw3ReviewTeam, accounts, "unknown", msg, members);
  console.log(chalk.green(` ${endow_id} - Done!`));
}

export async function testApproveInactiveEndowment(
  juno: SigningCosmWasmClient,
  apTeam: string,
  accounts: string,
  endowment_id: number,
): Promise<void> {
  process.stdout.write("AP Review Team approves an inactive Charity endowment");

  const res = await juno.queryContractSmart(accounts, { config: {} });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, apTeam, cw3, accounts, {
      update_endowment_status: {
        endowment_id,
        status: 1,
        beneficiary: undefined,
      }
    })
  ).to.be.ok;
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Can update an Endowment's status from the Accounts
//    Possible Status Values:
//    0. Inactive - NO Deposits | NO Withdraws - no beneficiary needed
//    1. Approved - YES Deposits | YES Withdraws - no beneficiary needed
//    2. Frozen - YES Deposits | NO Withdraws - no beneficiary needed
//    3. Closed - NO Deposits | NO Withdraws - IF beneficiary address given: funds go to that wallet
//                ELSE: sent to fund members
//----------------------------------------------------------------------------------------
export async function testUpdateEndowmentStatus(
  juno: SigningCosmWasmClient,
  apTeam: string,
  accounts: string,
  endowmentStatus: any, // { address: "juno1....", status: 0|1|2|3, benficiary: "juno1.." | undefined }
): Promise<void> {
  process.stdout.write("AP Team updates endowment's status");

  const res = await juno.queryContractSmart(accounts, { config: {} });
  const cw3 = res.owner as string;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, accounts, {
    update_endowment_status: {
      endowment_id: endowmentStatus.endowment_id,
      status: endowmentStatus.status,
      beneficiary: endowmentStatus.beneficiary,
    },
  });
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryAccountsState(
  juno: SigningCosmWasmClient,
  accountsContract: string,
  endowmentId: number,
): Promise<void> {
  process.stdout.write("Test - Query Accounts State");
  const result = await juno.queryContractSmart(accountsContract, {
    state: { id: endowmentId },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsTransactions(
  juno: SigningCosmWasmClient,
  accountsContract: string,
  sender: string | undefined,
  recipient: string | undefined,
  denom: string | undefined
): Promise<void> {
  process.stdout.write("Test - Query Accounts Transactions");
  const result = await juno.queryContractSmart(accountsContract, {
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
  accountsContract: string,
  endowmentId: number,
): Promise<void> {
  process.stdout.write(`Test - Query Accounts - Endowment(#${endowmentId}) Balance\n`);
  const result = await juno.queryContractSmart(accountsContract, {
    balance: { id: endowmentId },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(
  juno: SigningCosmWasmClient,
  accountsContract: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result = await juno.queryContractSmart(accountsContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowmentList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await juno.queryContractSmart(registrar, {
    endowment_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}


export async function testQueryAccountsEndowment(
  juno: SigningCosmWasmClient,
  accountsContract: string,
  endowmentId: number,
): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result = await juno.queryContractSmart(accountsContract, {
    endowment: { id: endowmentId },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsProfile(
  juno: SigningCosmWasmClient,
  accountsContract: string,
  endowmentId: number,
): Promise<void> {
  process.stdout.write("Test - Query Accounts Profile");
  const result = await juno.queryContractSmart(accountsContract, {
    get_profile: { id: endowmentId },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsTokenAmount(
  juno: SigningCosmWasmClient,
  accountsContract: string,
  endowmentId: number,
  asset_info: any,
  acct_type: any,
): Promise<void> {
  process.stdout.write("Test - Query Accounts Token Amount");
  const result = await juno.queryContractSmart(accountsContract, {
    token_amount: {
      id: endowmentId,
      asset_info,
      acct_type,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
