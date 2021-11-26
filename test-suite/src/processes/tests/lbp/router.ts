/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: swap operation
//
// SCENARIO:
//
//----------------------------------------------------------------------------------------
// export async function testRouterSwapOperation(
//   terra: LocalTerra | LCDClient,
//   apTeam: Wallet,
//   routerContract: string,
//   tokenContract: string,
// ): Promise<void> {
//   process.stdout.write("Test - SwapOperation");
//   await expect(
//     sendTransaction(terra, apTeam, [
//       new MsgExecuteContract(
//         apTeam.key.accAddress,
//         routerContract,
//         {
//           swap_operation: {
//             operations: [
//               {
//                 offer_asset_info: {
//                   token: {
//                     contract_addr: tokenContract
//                   }
//                 },
//                 ask_asset_info: {
//                   native_token: {
//                     denom: "uusd".toString()
//                   }
//                 }
//               }
//             ],
//             minimum_receive: undefined,
//             to: undefined,
//           },
//         },
//       ),
//     ])
//   );
//   console.log(chalk.green(" Passed!"));
// }

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
