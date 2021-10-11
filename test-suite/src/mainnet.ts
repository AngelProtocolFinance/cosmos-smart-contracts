/* eslint-disable @typescript-eslint/no-unused-vars */
import {Coin, LCDClient, MnemonicKey, Wallet} from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLCDClient,
  setupContractsForMainNet,
  testQueryAccountsBalance,
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
} from "./setup_mainnet";
import {initializeCharities, setupEndowments, createIndexFunds} from "./charities";
import dotenv from 'dotenv';

dotenv.config();
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LCDClient): Promise<void> {
  console.log(chalk.blue("\nColumbus-5 MainNet"));

  // CW4 AP Team Group Members
  const members = [
    {addr: "terra-wallet", weight: 1},
  ];

  // Add confirmed TCA Members to the Index Fund SCs approved list
  const tca_members: string[] = [];
  
  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContractsForMainNet(
    process.env.ap_treasury, // treasury address
    members, // CW4 AP Team Group Members
    tca_members, // confirmed TCA members
    "0.1",  // tax rate
    "0.50", // threshold absolute percentage
    200000,   // max voting period height
    100000,    // max voting period guardians height
    undefined, // index fund rotation
    false,   // turn over to AP Multisig
    "0.75",  // harvest to liquid percentage
    "0.0000000421740233", // tax_per_block: Anchor's 19.5% earnings collected per block
    "20000000000", // funding goal
  );

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  await testQueryRegistrarConfig();
  await testQueryRegistrarEndowmentList();
  await testQueryRegistrarApprovedVaultList();
  await testQueryRegistrarVaultList();
  await testQueryRegistrarVault();
  // await testQueryAccountsBalance();
  await testQueryIndexFundConfig();
  await testQueryIndexFundState();
  await testQueryIndexFundTcaList();
  await testQueryIndexFundFundsList();
  await testQueryIndexFundFundDetails();
  await testQueryIndexFundActiveFundDetails();
  await testQueryIndexFundActiveFundDonations();
}
