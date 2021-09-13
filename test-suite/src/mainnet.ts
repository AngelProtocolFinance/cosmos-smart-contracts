import {LCDClient, Coin, Wallet} from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLCDClient,
  setupContracts,
  testRejectUnapprovedDonations,
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testAngelTeamCanTriggerVaultsHarvest,
  testCharityCanUpdateStrategies,
  testBeneficiaryCanWithdrawFromLiquid,
} from "./main";
import dotenv from 'dotenv';

dotenv.config();
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LCDClient): Promise<void> {
  console.log(chalk.blue("\nColumbus-4 MainNet"));
  // get the current swap rate from 1 TerraUSD to TerraKRW
  console.log(chalk.yellow("\nStep1. Swap rate between uusd and uluna"));
  const offerCoin: Coin = new Coin("uusd", "1000000");
  const c: Coin = await terra.market.swapRate(offerCoin, "uluna");
  console.log(`${offerCoin.toString()} can be swapped for ${c.toString()}`);

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
  // await setupContracts();

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
  // await testQueryRegistrarApprovedEndowmentList();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarVaultList();
  // await testQueryRegistrarVault();
  // await testQueryAccountsBalance();
  // await testQueryAccountsConfig();
  // await testQueryAccountsEndowment();
  // await testQueryAccountsAccount();
  // await testQueryAccountsAccountList();
  // await testQueryIndexFundConfig();
  // await testQueryIndexFundState();
  // await testQueryIndexFundTcaList();
  // await testQueryIndexFundFundsList();
  // await testQueryIndexFundFundDetails();
  // await testQueryIndexFundActiveFundDetails();
  // await testQueryIndexFundActiveFundDonations();
}
