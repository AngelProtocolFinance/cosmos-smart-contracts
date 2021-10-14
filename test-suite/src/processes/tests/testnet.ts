import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment
} from "./accounts/test";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
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
  testAddApTeamMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner
} from "./multisig/test";
import {
  testAngelTeamCanTriggerVaultsHarvest,
  testClosingEndpoint,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
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
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  apTeam2: Wallet,
  apTeam3: Wallet,
  charity1: Wallet,
  charity2: Wallet,
  charity3: Wallet,
  pleb: Wallet,
  tca: Wallet,
  registrar: string,
  indexFund: string,
  anchorVault1: string,
  anchorVault2: string,
  endowmentContract1: string,
  endowmentContract2: string,
  endowmentContract3: string,
  endowmentContract4: string,
  cw4GrpApTeam: string,
  cw4GrpOwners: string,
  cw3ApTeam: string,
  cw3GuardianAngels: string
): Promise<void> {

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // Guardian angels multisig test
  await testAddApTeamMemberToC4Group(terra, apTeam, apTeam3, cw3ApTeam, cw4GrpApTeam);
  await testAddGuardiansToEndowment(terra, apTeam3, charity1, charity2, charity3, pleb, cw3GuardianAngels, endowmentContract1);
  // await testGuardiansChangeEndowmentOwner(terra, charity2, charity3, pleb, endowmentContract1, cw3GuardianAngels);
  // Test execute
  // await testRejectUnapprovedDonations(terra, pleb, endowmentContract3);
  // await testDonorSendsToIndexFund(terra, pleb, indexFund);
  // await testTcaMemberSendsToIndexFund(terra, tca, indexFund);
  // await testAngelTeamCanTriggerVaultsHarvest(terra, apTeam, charity1, registrar);
  // await testCharityCanUpdateStrategies(terra, charity1, endowmentContract1, anchorVault1, anchorVault2);
  // await testBeneficiaryCanWithdrawFromLiquid(terra, charity1, endowmentContract1, anchorVault1, anchorVault2);
  // await testUpdatingRegistrarConfigs(terra, apTeam, registrar);
  // await testClosingEndpoint(terra, apTeam, registrar, endowmentContract3, endowmentContract4);
  // await testMigrateAllAccounts(terra, apTeam, registrar);
  await testUpdateFundMembers(terra, apTeam, indexFund, endowmentContract2, endowmentContract4);
  // Test query
  await testQueryRegistrarConfig(terra, registrar);
  await testQueryRegistrarEndowmentList(terra, registrar);
  await testQueryRegistrarApprovedVaultList(terra, registrar);
  await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  await testQueryRegistrarVaultList(terra, registrar);
  await testQueryRegistrarVault(terra, registrar, anchorVault1);
  await testQueryAccountsBalance(terra, endowmentContract1);
  await testQueryVaultConfig(terra, anchorVault1);
  await testQueryAccountsConfig(terra, endowmentContract1);
  await testQueryAccountsEndowment(terra, endowmentContract1);
  await testQueryIndexFundConfig(terra, indexFund);
  await testQueryIndexFundState(terra, indexFund);
  await testQueryIndexFundTcaList(terra, indexFund);
  await testQueryIndexFundFundsList(terra, indexFund);
  await testQueryIndexFundFundDetails(terra, indexFund);
  await testQueryIndexFundActiveFundDetails(terra, indexFund);
  await testQueryIndexFundActiveFundDonations(terra, indexFund);
  await testQueryIndexFundDeposit(terra, indexFund);
}