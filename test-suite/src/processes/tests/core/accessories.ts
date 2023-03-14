/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendTransaction,
  sendTransactionWithFunds,
} from "../../../utils/juno/helpers";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

chai.use(chaiAsPromised);
const { expect } = chai;

export async function testSendDepositToGiftcards(
  juno: SigningCosmWasmClient,
  apTeam: string,
  giftcards: string,
  coin: any // { denom: "ujuno", amount: "100000" }
  // recipient: string | undefined,
): Promise<void> {
  process.stdout.write("Test - Send amount as a Deposit to Gift Card Contract");
  const res = await sendTransactionWithFunds(
    juno,
    apTeam,
    giftcards,
    { deposit: { to_address: undefined } },
    [coin]
  );
  console.log(chalk.green(" Passed!"));
}

export async function testClaimGiftcardsDeposit(
  juno: SigningCosmWasmClient,
  apTeam: string,
  giftcards: string,
  deposit: number,
  recipient: string
  // recipient: string | undefined,
): Promise<void> {
  process.stdout.write("Test - Claim a Deposit in Gift Card Contract");
  const res = await sendTransaction(juno, apTeam, giftcards, {
    claim: { deposit, recipient },
  });
  console.log(chalk.green(" Passed!"));
}

export async function testSpendGiftcardsBalance(
  juno: SigningCosmWasmClient,
  apTeam: string,
  giftcards: string,
  assset_denom: string,
  asset_amount: string,
  endow_id: number,
  locked_percentage: string,
  liquid_percentage: string
): Promise<void> {
  process.stdout.write("Test - Spend from a balance in Gift Card Contract");
  const res = await sendTransaction(juno, apTeam, giftcards, {
    spend: {
      asset: { info: { native: assset_denom }, amount: asset_amount },
      endow_id,
      locked_percentage,
      liquid_percentage,
    },
  });
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsBalance(
  juno: SigningCosmWasmClient,
  giftcards: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards Balance for some address");
  const result = await juno.queryContractSmart(giftcards, {
    balance: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsConfig(
  juno: SigningCosmWasmClient,
  giftcards: string
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards config");
  const result = await juno.queryContractSmart(giftcards, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsDeposit(
  juno: SigningCosmWasmClient,
  giftcards: string,
  deposit_id: number
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards Deposit record");
  const result = await juno.queryContractSmart(giftcards, {
    deposit: { deposit_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
