import * as fs from "fs";
import chalk from "chalk";
import BN from "bn.js";
import {
  isTxError,
  LocalTerra,
  Msg,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgStoreCode,
  Wallet,
  LCDClient,
} from "@terra-money/terra.js";
import axios from "axios";

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
  terra: LocalTerra | LCDClient,
  sender: Wallet,
  msgs: Msg[],
  verbose = false
) {
  let fee;
  try {
    let sequence = await sender.sequence();
    fee = await terra.tx.estimateFee([{ sequenceNumber: sequence }], { msgs });
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        chalk.red("Transaction failed!") +
          `\n${chalk.yellow("code")}: ${error.code}` +
          `\n${chalk.yellow("message")}: ${error.message}` +
          `\n${chalk.yellow("error")}: ${error.response?.data["error"]}`
      );
    } else {
      throw error;
    }
  }
  const tx = await sender.createAndSignTx({ msgs, fee });
  const result = await terra.tx.broadcast(tx);

  // Print the log info
  if (verbose) {
    console.log(chalk.magenta("\nTxHash:"), result.txhash);
    try {
      console.log(
        chalk.magenta("Raw log:"),
        JSON.stringify(JSON.parse(result.raw_log), null, 2)
      );
    } catch {
      console.log(chalk.magenta("Failed to parse log! Raw log:"), result.raw_log);
    }
  }

  if (isTxError(result)) {
    throw new Error(
      chalk.red("Transaction failed!") +
        `\n${chalk.yellow("code")}: ${result.code}` +
        `\n${chalk.yellow("codespace")}: ${result.codespace}` +
        `\n${chalk.yellow("raw_log")}: ${result.raw_log}`
    );
  }

  return result;
}

/**
 * @notice Upload contract code to LocalTerra. Return code ID.
 */
export async function storeCode(
  terra: LocalTerra | LCDClient,
  deployer: Wallet,
  filepath: string
): Promise<number> {
  const code = fs.readFileSync(filepath).toString("base64");
  const result = await sendTransaction(terra, deployer, [
    new MsgStoreCode(deployer.key.accAddress, code),
  ]);
  return parseInt(result.logs[0].eventsByType.store_code.code_id[0]);
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function instantiateContract(
  terra: LocalTerra | LCDClient,
  deployer: Wallet,
  admin: Wallet, // leave this emtpy then contract is not migratable
  codeId: number,
  instantiateMsg: Record<string, unknown>
) {
  const result = await sendTransaction(terra, deployer, [
    new MsgInstantiateContract(
      deployer.key.accAddress,
      admin.key.accAddress,
      codeId,
      instantiateMsg,
      undefined,
      "instantiate_contract",
    ),
  ]);
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function migrateContract(
  terra: LocalTerra | LCDClient,
  sender: Wallet,
  admin: Wallet,
  contract: string,
  new_code_id: number,
  migrateMsg: Record<string, unknown>
) {
  const result = await sendTransaction(terra, sender, [
    new MsgMigrateContract(admin.key.accAddress, contract, new_code_id, migrateMsg),
  ]);
  return result;
}

/**
 * @notice Instantiate a contract from an existing code ID. Return contract address.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export async function migrateContracts(
  terra: LocalTerra | LCDClient,
  sender: Wallet,
  admin: Wallet,
  contracts: string[],
  new_code_id: number,
  migrateMsg: Record<string, unknown>
) {
  let msgs: Msg[] = [];
  contracts.forEach((contract) => {
    msgs.push(
      new MsgMigrateContract(admin.key.accAddress, contract, new_code_id, migrateMsg)
    );
    console.log(`Endmowment ${contract} - ${chalk.green("Msg built")}`);
  });

  const result = await sendTransaction(terra, sender, msgs);
  console.log(result);
  return result;
}

/**
 * @notice Return the native token balance of the specified account
 */
// export async function queryNativeTokenBalance(
//   terra: LocalTerra | LCDClient,
//   account: string,
//   denom = "uusd"
// ): Promise<string> {
//   const balance = (await terra.bank.balance(account)).get(denom)?.amount.toString();
//   if (balance) {
//     return balance;
//   } else {
//     return "0";
//   }
// }

/**
 * @notice Return CW20 token balance of the specified account
 */
export async function queryTokenBalance(
  terra: LocalTerra | LCDClient,
  account: string,
  contract: string
): Promise<string> {
  const balanceResponse = await terra.wasm.contractQuery<{ balance: string }>(contract, {
    balance: { address: account },
  });
  return balanceResponse.balance;
}

/**
 * @notice Given a total amount of UST, find the deviverable amount, after tax, if we
 * transfer this amount.
 * @param amount The total amount
 * @dev Assumes a tax rate of 0.001 and cap of 1000000 uusd.
 * @dev Assumes transferring UST. Transferring LUNA does not incur tax.
 */
export function deductTax(amount: number): number {
  const DECIMAL_FRACTION = new BN("1000000000000000000");
  const tax = Math.min(
    amount -
      new BN(amount)
        .mul(DECIMAL_FRACTION)
        .div(DECIMAL_FRACTION.div(new BN(1000)).add(DECIMAL_FRACTION))
        .toNumber(),
    1000000
  );
  return amount - tax;
}

/**
 * @notice Given a intended deliverable amount, find the total amount, including tax,
 * necessary for deliver this amount. Opposite operation of `deductTax`.
 * @param amount The intended deliverable amount
 * @dev Assumes a tax rate of 0.001 and cap of 1000000 uusd.
 * @dev Assumes transferring UST. Transferring LUNA does not incur tax.
 */
export function addTax(amount: number): number {
  const tax = Math.min(new BN(amount).div(new BN(1000)).toNumber(), 1000000);
  return amount + tax;
}
