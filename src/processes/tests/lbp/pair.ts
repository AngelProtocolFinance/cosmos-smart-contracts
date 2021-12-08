/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, LocalTerra } from "@terra-money/terra.js";

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryPairPair(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pair");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pair: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairPool(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pool");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pool: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairSimulationNativeToHalo(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pair Simulation UST->HALO ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    simulation: {
      offer_asset: {
        info:{
          native_token: {
            denom: "uusd".toString()
          }
        },
        amount: "100000000"
      },
      block_time: Math.round(currTime)
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairSimulationHaloToNative(
  terra: LocalTerra | LCDClient,
  pairContract: string,
  tokenContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pair Simulation HALO->UST ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    simulation: {
      offer_asset: {
        info:{
          token: {
            contract_addr: tokenContract
          }
        },
        amount: "100000000"
      },
      block_time: Math.round(currTime)
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairReverseSimulationNativeToHalo(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pair Reverse Simulation UST -> HALO ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    reverse_simulation: {
      ask_asset: {
        info:{
          native_token: {
            denom: "uusd".toString()
          }
        },
        amount: "100000000"
      },
      block_time: Math.round(currTime)
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairReverseSimulationHaloToNative(
  terra: LocalTerra | LCDClient,
  pairContract: string,
  tokenContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pair Reverse Simulation HALO -> UST ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    reverse_simulation: {
      ask_asset: {
        info:{
          token: {
            contract_addr: tokenContract
          }
        },
        amount: "100000000"
      },
      block_time: Math.round(currTime)
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
