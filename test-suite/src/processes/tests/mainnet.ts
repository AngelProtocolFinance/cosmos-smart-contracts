import { LCDClient } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment
} from "./accounts/test";
import {
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundDeposit,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList
} from "./indexFunds/test";
import {
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList
} from "./registrar/test";
import {
  testQueryVaultConfig
} from "./vaults/test";

export async function testExecute(
  terra: LCDClient,
  registrar: string,
  indexFund: string,
  anchorVault: string,
  endowmentContract: string,
): Promise<void> {

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // Test query
  await testQueryRegistrarConfig(terra, registrar);
  await testQueryRegistrarEndowmentList(terra, registrar);
  await testQueryRegistrarApprovedVaultList(terra, registrar);
  await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  await testQueryRegistrarVaultList(terra, registrar);
  await testQueryRegistrarVault(terra, registrar, anchorVault);
  await testQueryAccountsBalance(terra, endowmentContract);
  await testQueryVaultConfig(terra, anchorVault);
  await testQueryAccountsConfig(terra, endowmentContract);
  await testQueryAccountsEndowment(terra, endowmentContract);
  await testQueryIndexFundConfig(terra, indexFund);
  await testQueryIndexFundState(terra, indexFund);
  await testQueryIndexFundTcaList(terra, indexFund);
  await testQueryIndexFundFundsList(terra, indexFund);
  await testQueryIndexFundFundDetails(terra, indexFund);
  await testQueryIndexFundActiveFundDetails(terra, indexFund);
  await testQueryIndexFundActiveFundDonations(terra, indexFund);
  await testQueryIndexFundDeposit(terra, indexFund);
}
