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

  // console.log(chalk.yellow("\nStep 1. Environment Info"));
  // initializeLCDClient(
  //   terra,
  //   {
  //     apTeam: wallet,
  //     charity1: wallet,
  //     charity2: wallet,
  //     charity3: wallet,
  //     pleb: wallet,
  //     tca: wallet
  //   },
  //   process.env.MONEYMARKET_CONTRACT_MAINNET
  // );

  // console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  // await setupContractsForMainNet(
  //   "treasuryAddress" // treasury address
  //   "0.2",  // tax rate
  //   "0.50", // threshold absolute percentage
  //   1000,   // max voting period height
  //   100,    // max voting period guardians height
  //   10,     // index fund rotation
  //   "0.75",  // harvest to liquid percentage
  //   "0.0000000259703196" // tax_per_block: 70% of Anchor's 19.5% earnings collected per block
  // );

  // console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // setTimeout(async () => {
  //   await testBeneficiaryCanWithdrawFromLiquid();
  // }, 7000);
  // await testQueryRegistrarConfig();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarVaultList();
  // await testQueryRegistrarVault();
  // await testQueryAccountsBalance();
  // await testQueryIndexFundConfig();
  // await testQueryIndexFundState();
  // await testQueryIndexFundTcaList();
  // await testQueryIndexFundFundsList();
  // await testQueryIndexFundFundDetails();
  // await testQueryIndexFundActiveFundDetails();
  // await testQueryIndexFundActiveFundDonations();
}
