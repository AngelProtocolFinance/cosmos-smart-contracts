import { LocalTerra } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLocalTerra,
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
export async function startTest(terra: LocalTerra): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initializeLocalTerra(terra);

  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContracts();

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  await testRejectUnapprovedDonations();
  await testDonorSendsToIndexFund();
  await testTcaMemberSendsToIndexFund();
  await testAngelTeamCanTriggerVaultsHarvest();
  await testCharityCanUpdateStrategies();
  setTimeout(async () => {
    await testBeneficiaryCanWithdrawFromLiquid();
  }, 500);
  await testQueryRegistrarConfig();
  await testQueryRegistrarApprovedEndowmentList();
  await testQueryRegistrarEndowmentList();
  await testQueryRegistrarApprovedVaultList();
  await testQueryRegistrarVaultList();
  await testQueryRegistrarVault();
  await testQueryAccountsBalance();
  await testQueryAccountsConfig();
  await testQueryAccountsEndowment();
  await testQueryAccountsAccount();
  await testQueryAccountsAccountList();
  await testQueryIndexFundConfig();
  await testQueryIndexFundState();
  await testQueryIndexFundTcaList();
  await testQueryIndexFundFundsList();
  await testQueryIndexFundFundDetails();
  await testQueryIndexFundActiveFundDetails();
  await testQueryIndexFundActiveFundDonations();
  
}
