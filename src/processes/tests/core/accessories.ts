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
  	let res = await sendTransactionWithFunds(juno, apTeam, giftcards, 
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
  recipient: string,
  // recipient: string | undefined,
): Promise<void> {
  	process.stdout.write("Test - Claim a Deposit in Gift Card Contract");
  	let res = await sendTransaction(juno, apTeam, giftcards, { 
  		claim: { deposit, recipient } 
  	});
  	console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsBalance(
  juno: SigningCosmWasmClient,
  giftcards: string,
  address: string,
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards Balance for some address");
  const result = await juno.queryContractSmart(giftcards, {
    Balance: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsConfig(
  juno: SigningCosmWasmClient,
  giftcards: string,
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards config");
  const result = await juno.queryContractSmart(giftcards, {
    Config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGiftcardsDeposit(
  juno: SigningCosmWasmClient,
  giftcards: string,
  deposit_id: number,
): Promise<void> {
  process.stdout.write("Test - Query Gift Cards Deposit record");
  const result = await juno.queryContractSmart(giftcards, {
    Deposit: { deposit_id },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
