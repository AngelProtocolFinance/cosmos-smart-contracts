/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: swap operation
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
export async function testRouterSwapOperations(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerContract: string,
  tokenContract: string,
  sender: string,
  amount: string,
): Promise<void> {
  process.stdout.write("Test - SwapOperation");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        routerContract,
        {
          receive: {
            sender,
            amount,
            msg: toEncodedBinary({
              execute_swap_operations: {
                operations: [
                  {
                    astro_swap: {
                      offer_asset_info: {
                        native_token: {
                          denom: "uusd",
                        }
                      },
                      ask_asset_info: {
                        token: {
                          contract_addr: tokenContract,
                        }
                      },
                    }
                  }
                ],
                minimum_receive: undefined,
                to: undefined,
              }
            }) 
          },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryRouterConfig(
  terra: LocalTerra | LCDClient,
  routerContract: string
): Promise<void> {
  process.stdout.write("Test - Query Router Config");
  const result: any = await terra.wasm.contractQuery(routerContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRouterSimulateSwapOperations(
  terra: LocalTerra | LCDClient,
  routerContract: string,
  tokenContract: string,
  offer_amount: string,
): Promise<void> {
  process.stdout.write("Test - Query Simulate Swap Operations");
  const currTime = new Date().getTime() / 1000;
  const result: any = await terra.wasm.contractQuery(routerContract, {
    simulate_swap_operations: {
      offer_amount,
      block_time: Math.round(currTime),
      operations: [
        { 
          astro_swap: {
            offer_asset_info: {
              native_token: {
                denom: "uusd",
              }
            },
            ask_asset_info: {
              token: {
                contract_addr: tokenContract,
              }
            },
          }
        }
      ]
    }
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
