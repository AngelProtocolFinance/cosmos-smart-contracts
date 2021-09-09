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
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LCDClient) {
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
  //   }
  // );

  // console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  // await setupContracts();

  // console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // await testBeneficiaryCanWithdrawFromLiquid();
}
