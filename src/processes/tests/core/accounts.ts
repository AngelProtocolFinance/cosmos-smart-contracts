/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction, sendTransactionWithFunds, sendMessageViaCw3Proposal, sendApplicationViaCw3Proposal, clientSetup, getWalletAddress } from "../../../utils/helpers";
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
  coin: any // { denom: "ujuno", amount: "100000" }
): Promise<void> {
  process.stdout.write("Test - Send amount to a single Endowment Account (50:50 split)");
  await expect(
    sendTransactionWithFunds(juno, apTeam, accountsContract, {
      deposit: {
        id: endowmentId,
        locked_percentage: "0.5",
        liquid_percentage: "0.5",
      },
    },
      [coin]
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

export async function testSendRestitutionFundsToEndowments(
  juno: SigningCosmWasmClient,
  wallet: string,
  accountsContract: string,
  endowments: any[],
  denom: string,
): Promise<void> {
  console.log(`Test - Send restitution funds to a new batch of charity endowments`);
  let prom = Promise.resolve();
  endowments.forEach((e) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            console.log(`Sending ${chalk.blue(`${e.amount} ${e.acct_type}`)} restitution funds to Endowment ID: ${chalk.blue(e.id)}`);
            let res = await sendTransactionWithFunds(juno, wallet, accountsContract, {
                deposit: {
                  id: e.id,
                  locked_percentage: (e.acct_type == "locked") ? "1" : "0",
                  liquid_percentage: (e.acct_type == "liquid") ? "1" : "0",
                },
              },
              [{ denom, amount: e.amount }]
            );
            console.log(chalk.green(" Sent!"));
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });
  await prom;
}

//----------------------------------------------------------------------------------------
// TEST: Endowment owner can withdraw from available balance in their Accounts
//
// SCENARIO:
// Endowment owner can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------
export async function testEndowmentCanWithdrawLiquid(
  juno: SigningCosmWasmClient,
  endowMember: string,
  accountsContract: string,
  endowmentId: number,
  beneficiary: string,
  assets: any,
): Promise<void> {
  process.stdout.write(
    "Test - Charity Owner can withdraw from the Endowment's liquid amount"
  );

  const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
  const cw3 = res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, endowMember, cw3, accountsContract, {
      withdraw: {
        id: endowmentId,
        acct_type: `liquid`,
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
// TEST: Charity Member of the Charity Endowment CW3 can send a proposal for a Locked Withdraw to the AP Team CW3.
// 
// Abstract away steps to send an Charity Endowment's Locked Withdraw proposal message to 
// the AP Team CW3 multisig for subsequent approval:
// 1. Create Early Locked Withdraw Proposal on Charity Endowment's CW3 to execute Locked Withdraw Proposal on AP Team CW3 contract
// 2. Capture the new Proposal's ID
// 3. Optional: Addtional Charity Endowment CW3 member(s) vote on the open poll
// 4. Proposal needs to be executed
// 5. Capture the new proposal ID from AP Team CW3
// 6. Optional: Addtional AP Team CW3 member(s) vote on the open poll 
// 7. Proposal on CW3 AP Team needs to be executed 
//----------------------------------------------------------------------------------------
export async function testCharityCanWithdrawLocked(
  networkUrl: string,
  proposor: DirectSecp256k1HdWallet,
  accountsContract: string,
  apTeamCw3: string,
  endowmentId: number,
  assets: any,
  endow_members: DirectSecp256k1HdWallet[],
  apteam_members: DirectSecp256k1HdWallet[],
): Promise<void> {
  process.stdout.write(
    "Test - Charity Member can withdraw from their Endowment's Locked account with AP Team Approval\n"
  );
  let proposor_client = await clientSetup(proposor, networkUrl);
  let proposor_wallet = await getWalletAddress(proposor);
  console.log(chalk.yellow(`> Charity ${proposor_wallet} submits an early withdraw proposal to their CW3`));
  
  // 0. Get the charity endowment's CW3 
  const res = await proposor_client.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
  const endowCw3 = res.owner as string;
  
  // 1. Create the new proposal (no vote is cast here)
  const proposal = await sendTransaction(proposor_client, proposor_wallet, endowCw3, {
    propose_locked_withdraw: {
        endowment_id: endowmentId,
        description: "SHOW ME THE MONEYYYY!",
        beneficiary: proposor_wallet, // send to the charity proposer's wallet
        assets
      },
  });

  // 2. Parse out the proposal ID
  const endowment_proposal_id = await proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;
  console.log(chalk.yellow(`> Endowment CW3's New Proposal's ID: ${endowment_proposal_id}`));

  // 3. Additional members need to vote on proposal to get to passing threshold
  let endow_prom = Promise.resolve();
  endow_members.forEach((member) => {
    // eslint-disable-next-line no-async-promise-executor
    endow_prom = endow_prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            let voter_wallet = await getWalletAddress(member);
            let voter_client = await clientSetup(member, networkUrl);
            console.log(chalk.yellow(`> Endowment CW3 Member ${voter_wallet} votes YES on endowment's proposal`));
            await sendTransaction(voter_client, voter_wallet, endowCw3, {
              vote: {
                proposal_id: parseInt(endowment_proposal_id),
                vote: `yes`,
              },
            });
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });
  await endow_prom;

  // 4. Execute the Endowment's CW3 Proposal (this creates an AP Team CW3 proposal)
  console.log(chalk.yellow("> Executing the Endowment CW3 Proposal"));
  const endowment_res = await sendTransaction(proposor_client, proposor_wallet, endowCw3, {
    execute: { proposal_id: parseInt(endowment_proposal_id) }
  });

  // 5. Capture and return the newly created AP Team CW3 Proposal ID (from returned submessage attr)
  let apteam_proposal_id = await endowment_res.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;
  console.log(chalk.yellow(`> AP Team CW3's New Proposal's ID: ${apteam_proposal_id}`));

  // 6. Voting on AP Team C3 proposal occurs
  let apteam_prom = Promise.resolve();
  apteam_members.forEach((member) => {
    // eslint-disable-next-line no-async-promise-executor
    apteam_prom = apteam_prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            let voter_wallet = await getWalletAddress(member);
            let voter_client = await clientSetup(member, networkUrl);
            console.log(chalk.yellow(`> AP Team CW3 Member ${voter_wallet} votes YES on AP Team proposal`));
            await sendTransaction(voter_client, voter_wallet, apTeamCw3, {
              vote: {
                proposal_id: parseInt(apteam_proposal_id),
                vote: `yes`,
              },
            });
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });
  await apteam_prom;

  // 7. Execute the AP Team CW3 proposal
  console.log(chalk.yellow("> Executing the AP Team CW3 Proposal"));
  const apteam_res = await sendTransaction(proposor_client, proposor_wallet, apTeamCw3, {
    execute: { proposal_id: parseInt(apteam_proposal_id) }
  });
  console.log(chalk.green(" Done!"));
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
export interface Strategy {
  vault: string; // Vault SC Address
  percentage: string; // percentage of funds to invest
}

export async function testCharityCanUpdateStrategies(
  juno: SigningCosmWasmClient,
  charity: string,
  accountsContract: string,
  endowmentId: number,
  acct_type: string,
  strategies: Strategy[], // [ { vault: string, percentage: "decimal" }, ... ]
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

export async function testCreateNormalEndowment(
  juno: SigningCosmWasmClient,
  apTeam: string,
  accounts: string,
  msg: any,
): Promise<void> {
  process.stdout.write("Create a new endowment via the CW3 Applications contract");
  let endow_res = await sendTransaction(juno, apTeam, accounts, { 
    create_endowment: msg
  });
  // capture and return the new Endowment ID
  let endow_id = await parseInt(endow_res.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
  console.log(chalk.green(`> New Endowment ID: ${endow_id} - Done!`));
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
    update_endowment_status: endowmentStatus
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
  console.log("Locked native:", result.tokens_on_hand.locked.native);
  console.log("Liquid native:", result.tokens_on_hand.liquid.native);
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
  accounts: string
): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment List");
  const result: any = await juno.queryContractSmart(accounts, {
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
  console.log("Locked strat:", result.strategies.locked);
  console.log("Liquid strat:", result.strategies.liquid);
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
