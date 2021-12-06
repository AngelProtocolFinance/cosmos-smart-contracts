/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { Coin, LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Swap
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairSwap(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  sender: string,
  amount: string,
): Promise<void> {
  process.stdout.write("Test - Swap");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        pairContract,
        {
          receive: {
            sender,
            amount,
            msg: toEncodedBinary({
              swap: {
                belief_price: undefined,
                max_spread: undefined,
                to: undefined,
              }
            })
          }
        }
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Swap
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testPairWithdrawLiquidity(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  sender: string,
  amount: string,
): Promise<void> {
  process.stdout.write("Test - Withdraw liquidity");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        pairContract,
        {
          receive: {
            sender,
            amount,
            msg: toEncodedBinary({
              withdraw_liquidity: {}
            })
          }
        }
      ),
    ])
  );  
  console.log(chalk.green(" Passed!"));
}

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

export async function testQueryPairSimulation(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Simulation");
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

export async function testQueryPairReverseSimulation(
  terra: LocalTerra | LCDClient,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Simulation");
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
