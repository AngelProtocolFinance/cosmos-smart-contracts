/* eslint-disable @typescript-eslint/no-unused-vars */
import {LCDClient, Coin, MnemonicKey, Wallet, MsgSend, BlockTxBroadcastResult} from "@terra-money/terra.js";
import chalk from "chalk";
import {
  initializeLCDClient,
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
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryVaultConfig,
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
  testClosingEndpoint,
  testUpdatingRegistrarConfigs,
  testQueryIndexFundDeposit,
  testQueryRegistrarApprovedVaultRateList,
} from "./main";
import dotenv from 'dotenv';

dotenv.config();
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LCDClient): Promise<void> {
  console.log(chalk.blue("\nBombay-12 TestNet"));

  // get wallets
  const apTeam: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.APTEAM}));
  const apTeam2: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.APTEAM2}));
  const apTeam3: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.APTEAM3}));
  const charity1: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY1}));
  const charity2: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY2}));
  const charity3: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY3}));
  const pleb: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.PLEB}));
  const tca: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.TCA}));

  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initializeLCDClient(
    terra,
    {
      apTeam,
      apTeam2,
      apTeam3,
      charity1,
      charity2,
      charity3,
      pleb,
      tca
    },
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    process.env.MONEYMARKET_CONTRACT_TESTNET!
  );

  // console.log(chalk.yellow("\nStep 2a. Migrate Contracts"));
  // await migrateContracts();
  // await testClosingEndpoint();

  // console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  // await setupContractsForTestNet(
  //   "0.2",  // tax rate
  //   "0.50", // threshold absolute percentage
  //   1000,   // max voting period height
  //   100,    // max voting period guardians height
  //   10,     // index fund rotation
  //   true,   // turn over to AP Team multisig
  //   false   // is LocalTerra
  // );

  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testAddApTeamMemberToC4Group();
  // await testAddGuardiansToEndowment();
  // await testGuardiansChangeEndowmentOwner();
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest("0.75");
  // setTimeout(async () => {
  //   await testCharityCanUpdateStrategies();
  // }, 8000);
  // setTimeout(async () => {
  //   await testBeneficiaryCanWithdrawFromLiquid();
  // }, 7000);
  // await testQueryRegistrarConfig();
  // await testUpdatingRegistrarConfigs();
  // await testQueryRegistrarApprovedEndowmentList();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarApprovedVaultRateList();
  // await testQueryRegistrarVaultList();
  // await testQueryRegistrarVault();
  // await testQueryAccountsBalance("terra1atuv3kzavg4q4nr4qt47zjquzjk84vl8lvl4tk");
  // await testQueryAccountsBalance("terra1yy9xmnvyssf6dfgh7xze8zl0cc5c37jxt53qf4");
  // await testQueryVaultConfig("terra15pa4v2kyr825tyyscygs8fygsctwv5qc7dqwcp");
  // await testQueryVaultConfig("terra1rtj2eg6cfm6fv6ldx0rasru26cck8dmwsyd4xa");
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
