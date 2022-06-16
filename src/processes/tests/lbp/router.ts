/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

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
  offer_amount: string
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
                denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
              },
            },
            ask_asset_info: {
              token: {
                contract_addr: tokenContract,
              },
            },
          },
        },
      ],
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
