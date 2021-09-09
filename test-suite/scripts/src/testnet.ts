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
  console.log(chalk.yellow("\nStep1. Swap rate between uusd and uluna"));
  const offerCoin: Coin = new Coin("uusd", "1000000");
  const c: Coin = await terra.market.swapRate(offerCoin, "uluna");
  console.log(`${offerCoin.toString()} can be swapped for ${c.toString()}`);

  // broadcasting transactions
  console.log(chalk.yellow("\nStep2. Broadcasting transactions"));
  // create a key out of a mnemonic
  const mk: MnemonicKey = new MnemonicKey({
    mnemonic: "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius",
  });

  // a wallet can be created out of any key wallets abstract transaction building
  const wallet: Wallet = terra.wallet(mk);

  // create a simple message that moves coin balances
  const send: MsgSend = new MsgSend(
    "terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v",
    "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    { uluna: 1000000, ukrw: 1230201, uusd: 1312029 }
  );

  const tx: StdTx = await wallet.createAndSignTx({ msgs: [send], memo: "test from terra.js!" });
  const result: BlockTxBroadcastResult = await terra.tx.broadcast(tx);
  console.log(`TX hash: ${result.txhash}`);


  // console.log(chalk.yellow("\nStep 1. Environment Info"));
  // initializeLCDClient(
  //   terra,
  //   {
  //     apTeam: wallet,
  //     charity1: wallet,
  //     charity2: wallet,
  //     charity3: wallet,
  //     pleb: wallet,
  //     tca: wallet
  //   },
  //   process.env.MONEYMARKET_CONTRACT_TESTNET
  // );

  // console.log(chalk.yellow("\nStep 2. Contracts Setup"));
  // await setupContracts();

  // console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testRejectUnapprovedDonations();
  // await testDonorSendsToIndexFund();
  // await testTcaMemberSendsToIndexFund();
  // await testAngelTeamCanTriggerVaultsHarvest();
  // await testCharityCanUpdateStrategies();
  // await testBeneficiaryCanWithdrawFromLiquid();
}
