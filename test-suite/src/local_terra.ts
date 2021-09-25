import { LocalTerra } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLocalTerra,
  setupContracts,
  migrateContracts,
  testAddApTeamMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner,
  testRejectUnapprovedDonations,
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testAngelTeamCanTriggerVaultsHarvest,
  testCharityCanUpdateStrategies,
  testBeneficiaryCanWithdrawFromLiquid,
  testMigrateAllAccounts,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList,
  testQueryRegistrarApprovedEndowmentList,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
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

  // console.log(chalk.yellow("\nStep 2b. Migrate Contracts"));
  // await migrateContracts();

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  await testAddApTeamMemberToC4Group();
  await testAddGuardiansToEndowment();
  await testGuardiansChangeEndowmentOwner();
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // setTimeout(async () => {
  //   await testBeneficiaryCanWithdrawFromLiquid();
  // }, 500);
  // await testQueryRegistrarConfig();
  // await testQueryRegistrarApprovedEndowmentList();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarVaultList();
  // await testQueryRegistrarVault();
  // await testQueryAccountsBalance();
  // await testQueryAccountsConfig();
  // await testQueryAccountsEndowment();
  // await testQueryIndexFundConfig();
  // await testQueryIndexFundState();
  // await testQueryIndexFundTcaList();
  // await testQueryIndexFundFundsList();
  // await testQueryIndexFundFundDetails();
  // await testQueryIndexFundActiveFundDetails();
  // await testQueryIndexFundActiveFundDonations();
}
