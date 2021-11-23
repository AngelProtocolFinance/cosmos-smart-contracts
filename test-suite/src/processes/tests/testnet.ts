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
  testUpdateAngelAllianceMembers,
  testUpdatingIndexFundConfigs,
  testRemoveIndexFund,
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
  await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateAngelAllianceMembers(terra, apTeam, indexFund, [
  //   "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // testnet admin (testnet ONLY!)
  //   "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf", // charity#1 (testnet ONLY!)
  //   "terra1zxtczmxtw8mk8xncvr8lcq2qmvk4dz88ek6f79", // community
  //   "terra18n2pc9x6q9str9dz8sqpt7ulz5telutclkzaec", // lunapes
  //   "terra17me29hk8cdd6mm6uf7cf0amsxmzxnszkfe5ph4", // lunabulls
  //   "terra1r59snugfm3gxjas565jf5ehw54junlfpmspjan", // lunabulls
  //   "terra1tz9jtxemq5e9sw048adz32tj62vkzp6f63e26f", // Astronorcs
  //   "terra1pl2cus25j79ukff04fxn9wwemerm2463gnztl6", // west coast
  //   "terra1etwq0q8wwnmq7322kz4v6ff2dcvwfm634vdkqn", // loop finance
  //   "terra157vv7nqra4zpfa58cglen5ekqmekxqw5ss3edq", // loop finance
  //   "terra1feqtlvaru4lszqnpjesgfw8splrg7u27wwwqac", // loop finance
  //   "terra1rzjxj4c6ykemk8csvtjchcqas7mul8s4w6rk8x", // tales of terra
  //   "terra1kf4k0l7hj5tlkuzf67ly43q8d2gcxay3hwa7fr", // hero
  //   "terra1yvg94g6ydgme2kdhy5t4gasgvem2kpk56g4h5e", // NFT Luna
  //   "terra14amh70rm5a3wjgkf7trvten3jfqum2svppky3e", // Terra Terrapins
  //   "terra1amrl8f5fqen2m478nuh2z7mz5ce096x4xqae9p", // Woof of Luna
  //   "terra1hxrd8pnqytqpelape3aemprw3a023wryw7p0xn", // ApolloDAO
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
}