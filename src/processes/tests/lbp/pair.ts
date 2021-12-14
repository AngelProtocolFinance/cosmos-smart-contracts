/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {
  Coin,
  LCDClient,
  LocalTerra,
  MsgExecuteContract,
  Wallet,
} from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Provide liquidity
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairProvideLiquidity(
  terra: LocalTerra | LCDClient,
  provider: Wallet,
  tokenContract: string,
  pairContract: string,
  tokenAmount: string,
  nativeTokenAmount: string
): Promise<void> {
  process.stdout.write("Provide liquidity to the New Pair contract");
  await sendTransaction(terra, provider, [
    new MsgExecuteContract(provider.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: tokenAmount,
        spender: pairContract,
      },
    }),
    new MsgExecuteContract(
      provider.key.accAddress,
      pairContract,
      {
        provide_liquidity: {
          assets: [
            {
              info: {
                token: {
                  contract_addr: tokenContract,
                },
              },
              amount: tokenAmount,
            },
            {
              info: {
                native_token: {
                  denom: "uusd",
                },
              },
              amount: nativeTokenAmount,
            },
          ],
        },
      },
      {
        uusd: nativeTokenAmount,
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Swap HALO -> Native
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairSwapHaloToNative(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  tokenContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Swap HALO -> Native ");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      send: {
        contract: pairContract,
        amount,
        msg: toEncodedBinary({
          swap: {},
        }),
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Swap HALO -> Native
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairSwapNativeToHalo(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  amount: string
): Promise<void> {
  process.stdout.write("Swap Native -> Halo ");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(
      apTeam.key.accAddress,
      pairContract,
      {
        swap: {
          sender: apTeam.key.accAddress,
          offer_asset: {
            info: {
              native_token: {
                denom: "uusd",
              },
            },
            amount,
          },
        },
      },
      [new Coin("uusd", amount)]
    ),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryPairPair(
  terra: LocalTerra | LCDClient,
  pairContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair\n");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pair: {},
  });

  console.log(`Asset Infos #1: ${JSON.stringify(result.asset_infos[0])}`);
  console.log(`Asset Infos #2: ${JSON.stringify(result.asset_infos[1])}`);
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairPool(
  terra: LocalTerra | LCDClient,
  pairContract: string
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
  pairContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair Simulation UST->HALO ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    simulation: {
      offer_asset: {
        info: {
          native_token: {
            denom: "uusd".toString(),
          },
        },
        amount: "100000000",
      },
      block_time: Math.round(currTime),
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairSimulationHaloToNative(
  terra: LocalTerra | LCDClient,
  pairContract: string,
  tokenContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair Simulation HALO->UST ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    simulation: {
      offer_asset: {
        info: {
          token: {
            contract_addr: tokenContract,
          },
        },
        amount: "100000000",
      },
      block_time: Math.round(currTime),
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairReverseSimulationNativeToHalo(
  terra: LocalTerra | LCDClient,
  pairContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair Reverse Simulation UST -> HALO ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    reverse_simulation: {
      ask_asset: {
        info: {
          native_token: {
            denom: "uusd".toString(),
          },
        },
        amount: "100000000",
      },
      block_time: Math.round(currTime),
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairReverseSimulationHaloToNative(
  terra: LocalTerra | LCDClient,
  pairContract: string,
  tokenContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair Reverse Simulation HALO -> UST ");
  const currTime = new Date().getTime() / 1000 + 10;
  const result: any = await terra.wasm.contractQuery(pairContract, {
    reverse_simulation: {
      ask_asset: {
        info: {
          token: {
            contract_addr: tokenContract,
          },
        },
        amount: "100000000",
      },
      block_time: Math.round(currTime),
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
