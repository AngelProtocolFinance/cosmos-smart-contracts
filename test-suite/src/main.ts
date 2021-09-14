/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
  storeCode,
  instantiateContract,
  migrateContract,
} from "./helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

let terra: LCDClient;
let apTeam: Wallet;
let charity1: Wallet;
let charity2: Wallet;
let charity3: Wallet;
let pleb: Wallet;
let tca: Wallet;

let accountsCodeId: number;
let registrar: string;
let indexFund: string;
let anchorVault1: string;
let anchorVault2: string;
let anchorMoneyMarket: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;

//----------------------------------------------------------------------------------------
// Initialize variables
//----------------------------------------------------------------------------------------
export function initializeLocalTerra(localTerra: LocalTerra): void {
  terra = localTerra;
  apTeam = localTerra.wallets.test1;
  charity1 = localTerra.wallets.test2;
  charity2 = localTerra.wallets.test3;
  charity3 = localTerra.wallets.test4;
  pleb = localTerra.wallets.test5;
  tca = localTerra.wallets.test6;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);
}

export function initializeLCDClient(
  lcdClient: LCDClient,
  wallets: {
    apTeam: Wallet,
    charity1: Wallet,
    charity2: Wallet,
    charity3: Wallet,
    pleb: Wallet,
    tca: Wallet
  },
  anchorMoneyMarketAddr: string): void {
  terra = lcdClient;
  apTeam = wallets.apTeam;
  charity1 = wallets.charity1;
  charity2 = wallets.charity2;
  charity3 = wallets.charity3;
  pleb = wallets.pleb;
  tca = wallets.tca;
  anchorMoneyMarket = anchorMoneyMarketAddr;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);
}

// -----------------------------
// Migrate Vault contracts
// -----------------------------
  export async function migrateContracts(): Promise<void> {
    process.stdout.write("Uploading Anchor Vault Wasm");
    const vaultCodeId = await storeCode(
      terra,
      apTeam,
      path.resolve(__dirname, "../../artifacts/anchor.wasm"));
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);
    
    // Anchor Vault - #1
    process.stdout.write("Migrate Anchor Vault (#1) contract");
    const vaultResult1 = await migrateContract(terra, apTeam, apTeam, anchorVault1, vaultCodeId, {});
    anchorVault1 = vaultResult1.logs[0].events.find((event) => {
      return event.type == "migrate_contract";
    })?.attributes.find((attribute) => { 
      return attribute.key == "contract_address"; 
    })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault1}`);

    // Anchor Vault - #2
    process.stdout.write("Migrate Anchor Vault (#2) contract");
    const vaultResult2 = await migrateContract(terra, apTeam, apTeam, anchorVault2, vaultCodeId, {});
    anchorVault2 = vaultResult2.logs[0].events.find((event) => {
      return event.type == "migrate_contract";
    })?.attributes.find((attribute) => { 
      return attribute.key == "contract_address"; 
    })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault2}`);
}

//----------------------------------------------------------------------------------------
// Setup all contracts
//----------------------------------------------------------------------------------------

export async function setupContracts(): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each.... 
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/registrar.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);
  
  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/anchor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);
  
  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/index_fund.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);
  
  process.stdout.write("Uploading Accounts Wasm");
  accountsCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/accounts.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);


  // Step 2. Instantiate the key contracts
  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(terra, apTeam, apTeam, registrarCodeId, {
    accounts_code_id: accountsCodeId,
    treasury: apTeam.key.accAddress,
    tax_rate: 20,
    default_vault: undefined,
  });
  registrar = registrarResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);
  
  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(terra, apTeam, apTeam, fundCodeId, {
    registrar_contract: registrar,
  });
  indexFund = fundResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);


  // Anchor Vault - #1
  process.stdout.write("Instantiating Anchor Vault (#1) contract");
  const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    name: "AP DP Token - Anchor #1",
    symbol: "apANC1",
    decimals: 6,
  });
  anchorVault1 = vaultResult1.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault1}`);

  // Anchor Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Anchor Vault (#2) contract");
  const vaultResult2 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    name: "AP DP Token - Anchor #2",
    symbol: "apANC",
    decimals: 6,
  });
  anchorVault2 = vaultResult2.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault2}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault #1 & #2 in Registrar");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: { 
        vault_addr: anchorVault1,
        approved: true,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: { 
        vault_addr: anchorVault2,
        approved: true,
      },
    })
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("Set default vault in Registrar (for newly created Endowments) as Anchor Vault #1");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        default_vault: anchorVault1,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
  
  // Step 4: Create two Endowments via the Registrar contract
  // endowment #1
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  const charityResult1 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity1.key.accAddress,
        beneficiary: charity1.key.accAddress,
        name: "Test Endowment #1",
        description: "A wonderful charity endowment that aims to test all the things",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract1 = charityResult1.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract1}`);

  // endowment #2
  process.stdout.write("Charity Endowment #2 created from the Registrar by the AP Team");
  const charityResult2 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity2.key.accAddress,
        beneficiary: charity2.key.accAddress,
        name: "Test Endowment #2",
        description: "An even better endowment full of butterflies and rainbows",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract2 = charityResult2.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract2}`);

  // endowment #3
  process.stdout.write("Charity Endowment #3 created from the Registrar by the AP Team");
  const charityResult3 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity3.key.accAddress,
        beneficiary: charity3.key.accAddress,
        name: "Test Endowment #3",
        description: "Shady endowment that will never be approved",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract3 = charityResult3.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract3}`);

  // AP Team approves 2 of 3 newly created endowments
  process.stdout.write("AP Team approves 2 of 3 endowments");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract1,
        status: 1,
      }
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract2,
        status: 1,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Step 5: Index Fund finals setup 
  // Update Index Fund Addr in the Registrar contract
  process.stdout.write("Update Registrar with the Address of the Index Fund contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        index_fund_contract: indexFund,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Create an initial Index Fund with the two charities created above
  process.stdout.write("Create an Index Fund with two charities in it");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: 1,
          name: "Test Fund",
          description: "My first test fund",
          members: [endowmentContract1, endowmentContract2],
        }
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Add confirmed TCA Members to the index fund approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_tca_list: { new_list: [tca.key.accAddress] },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}


//----------------------------------------------------------------------------------------
// TEST: Normal Donor cannot send funds to the Index Fund 
//
// SCENARIO:
// Normal user sends UST funds to an Index Fund SC fund to have it split 
// up amonst the fund's charity members. 
//
//----------------------------------------------------------------------------------------

export async function testDonorSendsToIndexFund(): Promise<void> {
  process.stdout.write("Test - Donor (normal pleb) cannot send a UST donation to an Index Fund fund");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(pleb.key.accAddress, indexFund,
        {
          deposit: {
            fund_id: 1,
            split: undefined,
          },
        },
        {
          uusd: "42000000",
        }
      ),
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate
  console.log(chalk.green("Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: Cannot send funds to an Endowment that is not approved for deposits 
//
// SCENARIO:
// If an Endowment has not been approved by the AP Team, all deposits should be rejected
//
//----------------------------------------------------------------------------------------

export async function testRejectUnapprovedDonations(): Promise<void> {
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
        {
          uusd: "42000000",
        }
      ),
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate
  console.log(chalk.green("Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: TCA Member can send donations to the Index Fund 
//
// SCENARIO:
// TCA Member sends UST funds to an Index Fund SC fund to have it split 
// up amonst the active fund's charity members. 
//
//----------------------------------------------------------------------------------------

export async function testTcaMemberSendsToIndexFund(): Promise<void> {
  process.stdout.write("Test - TCA Member can send a UST donation to an Index Fund");

  await expect(
    sendTransaction(terra, tca, [
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        {
          deposit: {
            fund_id: 1,
            split: undefined,
          },
        },
        {
          uusd: "400000000",
        }
      ),
    ])
  );
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team and trigger harvesting of Accounts for a Vault(s)
//
// SCENARIO:
// AP team needs to send a message to a Vault to trigger a rebalance of Endowment funds, 
// moving money from their Locked to Liquid & taking a small tax of DP Tokens as well.
//
//----------------------------------------------------------------------------------------
export async function testAngelTeamCanTriggerVaultsHarvest(): Promise<void> {
  process.stdout.write("Test - AP Team can trigger harvest of all Vaults (Locked to Liquid Account)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, registrar, {
        harvest: {}
      })
    ])
  ).to.be.rejectedWith("Unauthorized");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        harvest: {}
      })
    ])
  );

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
export async function testBeneficiaryCanWithdrawFromLiquid(): Promise<void> {
  process.stdout.write("Test - Beneficiary can withdraw from the Endowment availalble amount (liquid)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "5000000", liquid: "10000000"},
            {vault: anchorVault2, locked: "5000000", liquid: "10000000"}
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
            {vault: anchorVault1, locked: "0", liquid: "20000000"},
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

export async function testCharityCanUpdateStrategies(): Promise<void> {
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

export async function testQueryRegistrarConfig(): Promise<void> {
  process.stdout.write("Test - Query Registrar config and get proper result");
  const result: any = await terra.wasm.contractQuery(registrar, {
    config: {},
  });

  expect(result.owner).to.equal(apTeam.key.accAddress);
  expect(result.accounts_code_id).to.equal(accountsCodeId);
  expect(result.treasury).to.equal(apTeam.key.accAddress);
  expect(result.tax_rate).to.equal('0.02');
  expect(result.default_vault).to.equal(anchorVault1);
  expect(result.index_fund).to.equal(indexFund);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedEndowmentList(): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedEndowmentList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_endowment_list: {},
  });

  expect(result.endowments.length).to.equal(2);
  expect(result.endowments[0].address).to.equal(endowmentContract1);
  expect(result.endowments[0].status).to.equal('Approved');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentList(): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    endowment_list: {},
  });

  expect(result.endowments.length).to.equal(3);
  // TODO (borodanov): resolve possibility of different order of endowments
  // expect(result.endowments[0].address).to.equal(endowmentContract3);
  // expect(result.endowments[0].status).to.equal('Inactive');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultList(): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedVaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_list: {},
  });

  expect(result.vaults.length).to.equal(1);
  expect(result.vaults[0].address).to.equal(anchorVault1);
  expect(result.vaults[0].input_denom).to.equal('uusd');
  expect(result.vaults[0].yield_token).to.equal(registrar);
  expect(result.vaults[0].approved).to.equal(true);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVaultList(): Promise<void> {
  process.stdout.write("Test - Query Registrar VaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault_list: {},
  });

  expect(result.vaults.length).to.equal(1);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVault(): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault: {
      vault_addr: anchorVault1,
    },
  });

  expect(result.vault.address).to.equal(anchorVault1);
  expect(result.vault.input_denom).to.equal('uusd');
  expect(result.vault.yield_token).to.equal(registrar);
  expect(result.vault.approved).to.equal(true);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsBalance(): Promise<void> {
  process.stdout.write("Test - Query Accounts Balance");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    balance: {},
  });

  expect(result.balances.length).to.equal(2);
  expect(result.balances[0].denom).to.equal('uust');
  expect(result.balances[1].denom).to.equal('apANC');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    config: {},
  });

  expect(result.owner).to.equal(apTeam.key.accAddress);
  expect(result.registrar_contract).to.equal(registrar);
  expect(result.deposit_approved).to.equal(true);
  expect(result.withdraw_approved).to.equal(true);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowment(): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    endowment: {},
  });

  expect(result.owner).to.equal(charity1.key.accAddress);
  expect(result.beneficiary).to.equal(charity1.key.accAddress);
  expect(result.split_to_liquid.max).to.equal('1');
  expect(result.split_to_liquid.min).to.equal('0');
  expect(result.strategies.length).to.equal(1);
  expect(result.strategies[0].vault).to.equal(anchorVault1);
  expect(result.strategies[0].locked_percentage).to.equal('1');
  expect(result.strategies[0].liquid_percentage).to.equal('1');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsAccount(): Promise<void> {
  process.stdout.write("Test - Query Accounts Account");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    account: { account_type: 'locked' },
  });

  expect(result.account_type).to.equal('locked');
  expect(result.ust_balance).to.equal('0');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsAccountList(): Promise<void> {
  process.stdout.write("Test - Query Accounts AccountList");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    account_list: {},
  });

  expect(result.locked_account.account_type).to.equal('locked');
  expect(result.locked_account.ust_balance).to.equal('0');
  expect(result.liquid_account.account_type).to.equal('liquid');
  expect(result.liquid_account.ust_balance).to.equal('0');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundConfig(): Promise<void> {
  process.stdout.write("Test - Query IndexFund Config");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    config: {},
  });

  expect(result.owner).to.equal(apTeam.key.accAddress);
  expect(result.fund_rotation).to.equal(500000);
  expect(result.fund_member_limit).to.equal(10);
  expect(result.funding_goal).to.equal('0');

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundState(): Promise<void> {
  process.stdout.write("Test - Query IndexFund State");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    state: {},
  });

  expect(result.total_funds).to.equal(1);
  expect(result.active_fund).to.equal(1);
  expect(result.terra_alliance.length).to.equal(1);
  expect(result.terra_alliance[0]).to.equal(tca.key.accAddress);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundTcaList(): Promise<void> {
  process.stdout.write("Test - Query IndexFund TcaList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    tca_list: {},
  });

  expect(result.tca_members.length).to.equal(1);
  expect(result.tca_members[0]).to.equal(tca.key.accAddress);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundsList(): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundsList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    funds_list: {},
  });

  expect(result.funds.length).to.equal(1);
  expect(result.funds[0].id).to.equal(1);
  expect(result.funds[0].members.length).to.equal(2);
  expect(result.funds[0].members.includes(endowmentContract1));
  expect(result.funds[0].members.includes(endowmentContract2));

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundDetails(): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    fund_details: { fund_id: 1 },
  });

  expect(result.fund.id).to.equal(1);
  expect(result.fund.name).to.equal('Test Fund');
  expect(result.fund.members.length).to.equal(2);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDetails(): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_details: {},
  });

  expect(result.fund.id).to.equal(1);
  expect(result.fund.name).to.equal('Test Fund');
  expect(result.fund.members.length).to.equal(2);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDonations(): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDonations");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_donations: {},
  });

  expect(result.donors.length).to.equal(0);
  console.log(chalk.green("Passed!"));
}
