import * as fs from "fs";
import chalk from "chalk";
import BN from "bn.js";
import axios from "axios";
import { Coin } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { wasm_path } from "../config/wasmPaths";

export type Member = {
  addr: string;
  weight: number;
};

export type Actor = {
  addr: string;
  client: SigningCosmWasmClient;
  wallet: DirectSecp256k1HdWallet;
};

export async function getWalletAddress(wallet: DirectSecp256k1HdWallet) {
  let [account] = await wallet.getAccounts();
  return account.address;
}

/**
 * @notice Encode a JSON object to base64 binary
 */
export function toEncodedBinary(obj: any): string {
  return Buffer.from(JSON.stringify(obj)).toString("base64");
}

export function datetimeStringToUTC(date: string): number {
  try {
    return Math.round(Date.parse(date) / 1000);
  } catch (err) {
    throw "Date given is not parsable";
  }
}

/**
 * @notice Send a transaction. Return result if successful, throw error if failed.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function sendTransaction(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>,
  memo = undefined,
  verbose = false
) {
  try { 
    const result = await juno.execute(sender, contract, msg, "auto", memo, []);
    if (verbose) {
      console.log(chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~"));
      console.log(chalk.yellow(JSON.stringify(result.logs)));
    }
    return result;
  } catch (err: any) {
    throw new Error(`An error occured! | ${err.toString()}`);
  }
}

/**
 * @notice Send a transaction along with some amount of funds [Coin, ... ]. 
 * Returns result if successful, throw error if failed.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function sendTransactionWithFunds(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>,
  funds: Coin[],
  memo = undefined,
  verbose = false
) {
  try { 
    const result = await juno.execute(sender, contract, msg, "auto", memo, funds);
    if (verbose) {
      console.log(chalk.yellow("\n~~~ TX HASH: ", result.transactionHash, "~~~~"));
      console.log(chalk.yellow(JSON.stringify(result.logs)));
    }
    return result;
  } catch (err: any) {
    throw new Error(`An error occured! | ${err.toString()}`);
  }
}

/**
 * @notice Upload contract code to LocalJuno. Return code ID.
 */
export async function storeCode(
  juno: SigningCosmWasmClient,
  deployer: string,
  filepath: string
): Promise<number> {
  const code = fs.readFileSync(filepath);
  const result = await juno.upload(deployer, code, "auto");
  return result.codeId;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function instantiateContract(
  juno: SigningCosmWasmClient,
  deployer: string,
  admin: string, // leave this emtpy then contract is not migratable
  codeId: number,
  instantiateMsg: Record<string, unknown>
) {
  const result = await juno.instantiate(deployer, codeId, instantiateMsg, "instantiate", "auto", { admin: admin });
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function migrateContract(
  juno: SigningCosmWasmClient,
  sender: string,
  contract: string,
  new_code_id: number,
  migrateMsg: Record<string, unknown>
) {
  const result = await juno.migrate(sender, contract, new_code_id, migrateMsg, "auto");
  return result;
}

// --------------------------------------------------
// Wrapper Function:
// Stores wasm, gets new code, and migrates contract 
//---------------------------------------------------
export async function storeAndMigrateContract(
  juno: SigningCosmWasmClient,
  apTeam: string,
  contract: string,
  wasmFilename: string,
  msg = {}
): Promise<void> {
  process.stdout.write(`Uploading ${wasmFilename} Wasm`);
  const codeId = await storeCode(juno, apTeam, `${wasm_path.core}/${wasmFilename}`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write(`Migrate ${wasmFilename} contract`);
  const result = await migrateContract(juno, apTeam, contract, codeId, msg);
  console.log(chalk.green(" Done!"));
}
