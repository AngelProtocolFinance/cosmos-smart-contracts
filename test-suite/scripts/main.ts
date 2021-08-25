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
const pleb = terra.wallets.test4;

let accountsCodeId: number;
let registrar: string;
let indexFund: string;
let anchorVault: string;
// let anchorMoneyMarket: string;
let endowmentContract1: string;
let endowmentContract2: string;

//----------------------------------------------------------------------------------------
// Setup
//----------------------------------------------------------------------------------------

async function setupTest() {
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
    tax_rate: 2,
    default_vault: undefined,
  });
  registrar = registrarResult.logs[0].events[0].attributes[3].value;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(terra, apTeam, apTeam, fundCodeId, {
    registrar_contract: registrar,
  });
  indexFund = fundResult.logs[0].events[0].attributes[3].value;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);


  // Anchor Vault
  process.stdout.write("Instantiating Anchor Vault contract");
  const vaultResult = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    name: "AP DP Token - Anchor",
    symbol: "apANCHOR",
    decimals: 6,
  });
  let event = vaultResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  });
  anchorVault = event?.attributes[3].value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault}`);

  // Step 3: Create two Endowments via the Registrar contract
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


`  // // AP Team approves both newly created endowments
  // process.stdout.write("AP Team approves both endowments");
  // await sendTransaction(terra, apTeam, [
  //   new MsgExecuteContract(apTeam.key.accAddress, registrar, {
  //     update_endowment_status: {
  //       endowment_addr: endowmentContract1,
  //       status: 1,
  //     }
  //   }),
  //   new MsgExecuteContract(apTeam.key.accAddress, registrar, {
  //     update_endowment_status: {
  //       endowment_addr: endowmentContract2,
  //       status: 1,
  //     }
  //   }),
  // ]);
  // console.log(chalk.green(" Done!"));`
  

  // Step 4: Index Fund finals setup 
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
          id: 42,
          name: "Test Fund",
          description: "My first test fund",
          members: [endowmentContract1, endowmentContract2],
        }
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

}


//----------------------------------------------------------------------------------------
// Test 1. Normal Donor can send funds to the Index Fund 
//
// SCENARIO:
// Normal user sends UST funds to an Index Fund SC fund to have it split 
// up amonst the fund's charity members. 
//
//----------------------------------------------------------------------------------------

async function testDonorSendsToIndexFund() {
  process.stdout.write("\nTest - Donor (normal pleb) can send a UST donation to an Index Fund fund");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(pleb.key.accAddress, indexFund, {
        deposit: {
          fund_id: 42,
          split: undefined,
        },
        funds: { uusd: "420000000" },
      })
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate

  const fundTotal = await queryNativeTokenBalance(terra, indexFund, "uusd");
  expect(fundTotal).to.equal("0");
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------

(async () => {
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as angel team`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as charity #2`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as evil pleb`);

  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupTest();

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testDonorSendsToIndexFund();
})();
