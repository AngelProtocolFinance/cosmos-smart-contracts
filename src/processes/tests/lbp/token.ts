/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, LocalTerra } from "@terra-money/terra.js";

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryTokenBalance(
  terra: LocalTerra | LCDClient,
  tokenContract: string,
  address: string,
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
  tokenContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Token Info");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    token_info: {}
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryTokenMinter(
  terra: LocalTerra | LCDClient,
  tokenContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Token minter");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    minter: {}
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryTokenMarketingInfo(
  terra: LocalTerra | LCDClient,
  tokenContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Marketing info");
  const result: any = await terra.wasm.contractQuery(tokenContract, {
    marketing_info: {}
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
