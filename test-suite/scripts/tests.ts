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
// Variables Loaded From Local Config ts file
//----------------------------------------------------------------------------------------

const terra = new LocalTerra();
const apTeam = terra.wallets.test1;
const charity1 = terra.wallets.test2;
const charity2 = terra.wallets.test3;
const charity3 = terra.wallets.test4;
const pleb = terra.wallets.test5;
const tca = terra.wallets.test6;

import { contracts, accountsCodeId } from "../localConfig";

let registrar: string = contracts.registrar;
let indexFund: string = contracts.indexFund;
let anchorVault: string = contracts.anchorVault;
let endowmentContract1: string = contracts.endowmentContract1;
let endowmentContract2: string = contracts.endowmentContract2;
let endowmentContract3: string = contracts.endowmentContract3;

//----------------------------------------------------------------------------------------
// Test 1. Normal Donor cannot send funds to the Index Fund 
//
// SCENARIO:
// Normal user sends UST funds to an Index Fund SC fund to have it split 
// up amonst the fund's charity members. 
//
//----------------------------------------------------------------------------------------

async function testDonorSendsToIndexFund() {
  process.stdout.write("Test - Donor (normal pleb) cannot send a UST donation to an Index Fund fund");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        indexFund,
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
// Test 2. Cannot send funds to an Endowment that is not approved for deposits 
//
// SCENARIO:
// If an Endowment has not been approved by the AP Team, all deposits should be rejected
//
//----------------------------------------------------------------------------------------

async function testRejectUnapprovedDonations() {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        endowmentContract3,
        {
          deposit: {
            locked_percentage: "0.5",
            liquid_percentage: "0.5",
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
// Test 3. TCA Member can send donations to the Index Fund 
//
// SCENARIO:
// TCA Member sends UST funds to an Index Fund SC fund to have it split 
// up amonst the active fund's charity members. 
//
//----------------------------------------------------------------------------------------

async function testTcaMemberSendsToIndexFund() {
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
          uusd: "4200000000",
        }
      ),
    ])
  );
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// Test XX. Charity Owner can rebalance their portfolio/update the Accounts' strategy
//
// SCENARIO:
// Charity Owner can trigger a rebalance of their Accounts, which should: 
// 1) redeem all invested funds from Vaults to the Accounts
// 2) reinvest all redeemed funds, according the accounts' strategy
//
//----------------------------------------------------------------------------------------


//----------------------------------------------------------------------------------------
// Test XX. Charity Beneficiary can withdraw from available balance in their Accounts
//
// SCENARIO:
// Charity beneficiary can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------



//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------

(async () => {
  console.log(chalk.yellow("\nRunning Tests"));
  await testRejectUnapprovedDonations();
  await testDonorSendsToIndexFund();
  await testTcaMemberSendsToIndexFund();
})();
