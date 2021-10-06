import { LocalTerra } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLocalTerra,
  setupContractsForTestNet,
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
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
  testQueryIndexFundDeposit,
  testQueryRegistrarApprovedVaultRateList,
} from "./main";
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LocalTerra): Promise<void> {
  console.log(chalk.blue("\nLocalTerra"));
  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initializeLocalTerra(terra);

  console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  await setupContractsForTestNet(
    "0.2",  // tax rate
    "0.50", // threshold absolute percentage
    1000,   // max voting period height
    100,    // max voting period guardians height
    10,     // index fund rotation
    true,   // turn over to AP Team multisig
    true,   // is LocalTerra
  );

  // console.log(chalk.yellow("\nStep 2b. Migrate Contracts"));
  // await migrateContracts();

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testAddApTeamMemberToC4Group();
  // await testAddGuardiansToEndowment();
  // await testGuardiansChangeEndowmentOwner();
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // setTimeout(async () => {
  //   await testBeneficiaryCanWithdrawFromLiquid();
  // }, 500);
  // await testQueryRegistrarConfig();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarApprovedVaultRateList();
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
  // await testQueryIndexFundDeposit();
}
