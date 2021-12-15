/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, LocalTerra, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

//----------------------------------------------------------------------------------------
// TEST: Withdraw liquidity
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairWithdrawLiquidity(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  liquidityToken: string,
  amount: string
): Promise<void> {
  process.stdout.write("Withdraw liquidity token");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, liquidityToken, {
      send: {
        contract: pairContract,
        amount,
        msg: toEncodedBinary({
          withdraw_liquidity: {},
        }),
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryTokenBalance(
  terra: LocalTerra | LCDClient,
  tokenContract: string,
  address: string
): Promise<void> {
  process.stdout.write("Test - Query Token balance");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    balance: { address },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryTokenInfo(
  terra: LocalTerra | LCDClient,
  tokenContract: string
): Promise<void> {
  process.stdout.write("Test - Query Token Info");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    token_info: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryTokenMinter(
  terra: LocalTerra | LCDClient,
  tokenContract: string
): Promise<void> {
  process.stdout.write("Test - Query Token minter");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    minter: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
