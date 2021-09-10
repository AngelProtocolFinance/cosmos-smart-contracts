import {LCDClient, Coin, MnemonicKey, Wallet, MsgSend, StdTx, BlockTxBroadcastResult} from "@terra-money/terra.js";
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
import dotenv from 'dotenv';

dotenv.config();
//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------
export async function startTest(terra: LCDClient): Promise<void> {
  console.log(chalk.blue("\nBombay-10 TestNet"));

  // get the current swap rate from 1 TerraUSD to TerraKRW
  // console.log(chalk.yellow("\nStep1. Swap rate between uusd and uluna"));
  // const offerCoin: Coin = new Coin("uusd", "1000000");
  // const c: Coin = await terra.market.swapRate(offerCoin, "uluna");
  // console.log(`${offerCoin.toString()} can be swapped for ${c.toString()}`);

  // get wallets
  // console.log(chalk.yellow("\nStep2. Broadcasting transactions"));
  const apTeam: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.APTEAM}));
  const charity1: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY1}));
  const charity2: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY2}));
  const charity3: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.CHARITY3}));
  const pleb: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.PLEB}));
  const tca: Wallet = terra.wallet(new MnemonicKey({mnemonic: process.env.TCA}));

  // const tx: StdTx = await wallet.createAndSignTx({ msgs: [send], memo: "test from terra.js!" });
  // const result: BlockTxBroadcastResult = await terra.tx.broadcast(tx);
  // console.log(`TX hash: ${result.txhash}`);

  console.log(chalk.yellow("\nStep 1. Environment Info"));
  initializeLCDClient(
    terra,
    {
      apTeam,
      charity1,
      charity2,
      charity3,
      pleb,
      tca
    },
    process.env.MONEYMARKET_CONTRACT_TESTNET!
  );

  // console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  // await setupContracts();

  // console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // setTimeout(async () => {
  //   await testBeneficiaryCanWithdrawFromLiquid();
  // }, 7000);
  // await testQueryRegistrarConfig();
  // await testQueryRegistrarApprovedEndowmentList();
  // await testQueryRegistrarEndowmentList();
  // await testQueryRegistrarApprovedVaultList();
  // await testQueryRegistrarVaultList();
  // await testQueryRegistrarVault();
  // await testQueryAccountsBalance();
  // await testQueryAccountsConfig();
  // await testQueryAccountsEndowment();
  // await testQueryAccountsAccount();
  // await testQueryAccountsAccountList();
  // await testQueryIndexFundConfig();
  // await testQueryIndexFundState();
  // await testQueryIndexFundTcaList();
  // await testQueryIndexFundFundsList();
  // await testQueryIndexFundFundDetails();
  // await testQueryIndexFundActiveFundDetails();
  // await testQueryIndexFundActiveFundDonations();
}
