import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAllianceMembersList,
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundDeposit,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList
} from "./core/indexFunds";
import {
  testAddApTeamMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner
} from "./core/multisig";
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
} from "./core/registrar";
import {
  testQueryVaultConfig
} from "./core/vaults";
import {
  testAirdropClaim,
  testAirdropRegisterNewMerkleRoot,
  testAirdropUpdateConfig,
  testQueryAirdropMerkleRoot,
  testQueryAirdropConfig,
  testQueryAirdropIsClaimed,
  testQueryAirdropLatestStage
} from "./halo/airdrop";
import {
  testCollectorUpdateConfig,
  testCollectorSweep,
  testQueryCollectorConfig
} from "./halo/collector"

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
  cw3GuardianAngels: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
): Promise<void> {

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdateAllianceMembersList(terra, apTeam, indexFund, [
  //   "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // AP Wallet - Community
  //   "terra18n2pc9x6q9str9dz8sqpt7ulz5telutclkzaec", // LunApe
  // ]);
  // Guardian angels multisig test
  // await testAddApTeamMemberToC4Group(terra, apTeam, apTeam3, cw3ApTeam, cw4GrpApTeam);
  // await testAddGuardiansToEndowment(terra, apTeam3, charity1, charity2, charity3, pleb, cw3GuardianAngels, endowmentContract1);
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
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 2, [endowmentContract2], [endowmentContract4]);
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault1);
  // await testQueryAccountsBalance(terra, endowmentContract1);
  // await testQueryVaultConfig(terra, anchorVault1);
  // await testQueryAccountsConfig(terra, endowmentContract1);
  // await testQueryAccountsEndowment(terra, endowmentContract1);
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(terra, apTeam, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(terra, apTeam, haloAirdrop, "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37");
  // await testAirdropClaim(terra, apTeam, haloAirdrop);
  // await testQueryAirdropConfig(terra, haloAirdrop);
  // await testQueryAirdropMerkleRoot(terra, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(terra, haloAirdrop, 1, "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(terra, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(terra, apTeam, pleb, haloGov, haloCollector, "0.5");
  // await testCollectorSweep(terra, apTeam, haloCollector);
  // await testQueryCollectorConfig(terra, haloCollector);

}