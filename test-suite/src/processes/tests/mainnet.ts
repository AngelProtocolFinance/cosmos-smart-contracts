import { LCDClient, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment
} from "./accounts/test";
import {
  testUpdatingIndexFundConfigs,
  testUpdateFundMembers,
  testRemoveIndexFund,
  testUpdateAngelAllianceMembers,
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
  apTeam: Wallet,
  registrar: string,
  indexFund: string,
  anchorVault: string,
  endowmentContract: string,
): Promise<void> {

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateFundMembers(terra, apTeam, indexFund, 1, [
  //     "terra1d63nva4f7fdzlq5pnvs2wy3wkuh2qlaj77xpzr", // lay
  //     "terra19yy0g7jawqcfsh78e0zvywyagzezrjdhtuf94s", // foodbank
  //     "terra1aj240zlu6pg4yj6t2zqa6zu9dz5n5ez829cz22", // impact 
  //     "terra1f5d62v4965gk3jvtffle6g722khgmt669dwmst", // childrenreach
  //     "terra1ah48pqeyrue2rrmfum094vqc3v7w7dhgy3zama" // riz
  // ], []);
  // await testRemoveIndexFund(terra, apTeam, indexFund, 3);
  await testUpdateAngelAllianceMembers(terra, apTeam, indexFund, [
    "terra1zxtczmxtw8mk8xncvr8lcq2qmvk4dz88ek6f79", // community
    "terra18n2pc9x6q9str9dz8sqpt7ulz5telutclkzaec", // lunapes
    "terra17me29hk8cdd6mm6uf7cf0amsxmzxnszkfe5ph4", // lunabulls
    "terra1r59snugfm3gxjas565jf5ehw54junlfpmspjan", // lunabulls
    "terra1tz9jtxemq5e9sw048adz32tj62vkzp6f63e26f", // Astronorcs
    "terra1pl2cus25j79ukff04fxn9wwemerm2463gnztl6", // west coast
    "terra1etwq0q8wwnmq7322kz4v6ff2dcvwfm634vdkqn", // loop finance
    "terra157vv7nqra4zpfa58cglen5ekqmekxqw5ss3edq", // loop finance
    "terra1rzjxj4c6ykemk8csvtjchcqas7mul8s4w6rk8x", // tales of terra
    "terra1kf4k0l7hj5tlkuzf67ly43q8d2gcxay3hwa7fr", // hero
    "terra1yvg94g6ydgme2kdhy5t4gasgvem2kpk56g4h5e", // Luna Millionares Portrait
    "terra14amh70rm5a3wjgkf7trvten3jfqum2svppky3e", // Terra Terrapins
    "terra1amrl8f5fqen2m478nuh2z7mz5ce096x4xqae9p", // Woof of Luna
    "terra1hxrd8pnqytqpelape3aemprw3a023wryw7p0xn", // ApolloDAO
  ]);
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault);
  // await testQueryAccountsBalance(terra, endowmentContract);
  // await testQueryVaultConfig(terra, anchorVault);
  // await testQueryAccountsConfig(terra, endowmentContract);
  // await testQueryAccountsEndowment(terra, endowmentContract);
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);
}
