import * as path from "path";
import BN from "bn.js";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LocalTerra, MsgExecuteContract } from "@terra-money/terra.js";
import {
  toEncodedBinary,
  sendTransaction,
  storeCode,
  instantiateContract,
  queryNativeTokenBalance,
  queryTokenBalance,
} from "./helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

const terra = new LocalTerra();
const apTeam = terra.wallets.test1;
const charity1 = terra.wallets.test2;
const charity2 = terra.wallets.test3;
const charity3 = terra.wallets.test4;
const pleb = terra.wallets.test5;
const tca = terra.wallets.test6;

let accountsCodeId: number;
let registrar: string;
let indexFund: string;
let anchorVault1: string;
let anchorVault2: string;
// let anchorMoneyMarket: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;

//----------------------------------------------------------------------------------------
// Setup all contracts
//----------------------------------------------------------------------------------------

export async function setupContracts() {
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
    moneymarket: registrar, // placeholder addr for now
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
    moneymarket: registrar, // placeholder addr for now
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
        index_fund_contract: indexFund,
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

export async function testDonorSendsToIndexFund() {
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
          uusd: "420000000",
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

export async function testRejectUnapprovedDonations() {
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
          uusd: "420000000",
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

export async function testTcaMemberSendsToIndexFund() {
  process.stdout.write("Test - TCA Member can send a UST donation to an Index Fund");

  await expect(
    sendTransaction(terra, tca, [
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        {
          deposit: {
            // fund_id: 1,
            // split: undefined,
          },
        },
        {
          uusd: "4000000000",
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
export async function testAngelTeamCanTriggerVaultHarvest() {
  process.stdout.write("Test - AP Team can trigger Vault to harvest from Locked to Liquid Account");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, anchorVault1, {
        harvest: {}
      })
    ])
  ).to.be.rejectedWith("Unauthorized");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, anchorVault1, {
        harvest: {}
      }),
      new MsgExecuteContract(apTeam.key.accAddress, anchorVault2, {
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
export async function testBeneficiaryCanWithdrawFromLiquid() {
  process.stdout.write("Test - Beneficiary can withdraw from the Endowment availalble amount (liquid)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "50000000", liquid: "100000000"},
            {vault: anchorVault2, locked: "50000000", liquid: "100000000"}
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
            {vault: anchorVault1, locked: "0", liquid: "200000000"},
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

export async function testCharityCanUpdateStrategies() {
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
// Main
//----------------------------------------------------------------------------------------

(async () => {
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);

  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContracts();

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  await testRejectUnapprovedDonations();
  await testDonorSendsToIndexFund();
  await testTcaMemberSendsToIndexFund();
  await testAngelTeamCanTriggerVaultHarvest();
  // await testCharityCanUpdateStrategies();
  // await testBeneficiaryCanWithdrawFromLiquid();
})();
