/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LocalTerra, LCDClient, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, storeCode } from "../../utils/helpers";

// Deploy LBP contracts
export async function setupLBP(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  tokenCodeId: number,
  tokenContract: string,
  collector_addr: string,
  commission_rate: string,
  ): Promise<void> {
  process.stdout.write("Uploading LBP factory Wasm");
  const factoryCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_factory.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading LBP pair Wasm");
  const pairCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_pair.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading LBP router Wasm");
  const routerCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/lbp_router.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // Factory contract
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
    owner: apTeam.key.accAddress,
    collector_addr,
    commission_rate,
  });
  const factoryContract = factoryResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${factoryContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Factory contract");
  const currTime = new Date().getTime() / 1000 + 100;
  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      create_pair: {
        asset_infos: [
          {
            info:{
              token: {
                contract_addr: tokenContract,
              }
            },
            start_weight: "1",
            end_weight: "1"
          },
          {
            info:{
              native_token: {
                denom: "uusd".toString()
              }
            },
            start_weight: "1",
            end_weight: "1"
          }
        ],
        start_time: Math.round(currTime),
        end_time: Math.round(currTime) + 3600 * 24 * 3,
        description: undefined
      }
    })
  ]);

  const pairContract = pairResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  // Router contract
  process.stdout.write("Instantiating LBP Router contract");
  const routerResult = await instantiateContract(terra, apTeam, apTeam, routerCodeId, {
    halo_factory: factoryContract
  });
  const routerContract = routerResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${routerContract}`);

  process.stdout.write("Provide liquidity to the New Pair contract");
  const liqAddResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: "160000000000",
        spender: pairContract,
      },
    }),
    new MsgExecuteContract(
      apTeam.key.accAddress,
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
              amount: "160000000000",
            },
            {
              info: {
                native_token: {
                  denom: "uusd",
                },
              },
              amount: "2600000000",
            },
          ],
        },
      },
      {
        uusd: "2600000000",
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));
}
