/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update Factory config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testFactoryUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  token_code_id: number | undefined,
  pair_code_id: number | undefined
): Promise<void> {
  process.stdout.write("Test - Only owner can update Factory config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
        update_config: {
          token_code_id,
          pair_code_id,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Create Pair
//
// SCENARIO:
// Only the owner can execute it to create swap pair
//----------------------------------------------------------------------------------------
export async function testFactoryCreatePair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  tokenContract: string,
  denom: string,
  start_time: number,
  end_time: number | undefined,
  token_start_weight: string,
  token_end_weight: string,
  native_start_weight: string,
  native_end_weight: string,
  description: string | undefined
): Promise<string> {
  process.stdout.write("Test - Only the owner can execute it to create swap pair");

  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      create_pair: {
        asset_infos: [
          {
            info: {
              token: {
                contract_addr: tokenContract,
              },
            },
            start_weight: token_start_weight,
            end_weight: token_end_weight,
          },
          {
            info: {
              native_token: {
                denom: denom,
              },
            },
            start_weight: native_start_weight,
            end_weight: native_end_weight,
          },
        ],
        start_time,
        end_time,
        description,
      },
    }),
  ]);

  const pairContract = pairResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  console.log(chalk.green(" Passed!"));
  return pairContract;
}

//----------------------------------------------------------------------------------------
// TEST: remove from list of pairs
//
// SCENARIO:
// Factory Pair owner can remote from list of pairs
//
//----------------------------------------------------------------------------------------
export async function testFactoryUnregister(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  tokenContract: string,
  denom: string
): Promise<void> {
  process.stdout.write("Test - Factory Pair owner can remove from list of pairs");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
        unregister: {
          asset_infos: [
            {
              token: {
                contract_addr: tokenContract,
              },
            },
            {
              native_token: {
                denom: denom,
              },
            },
          ],
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryFactoryConfig(
  terra: LocalTerra | LCDClient,
  factoryContract: string
): Promise<void> {
  process.stdout.write("Test - Query Factory Config");
  const result: any = await terra.wasm.contractQuery(factoryContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryFactoryPair(
  terra: LocalTerra | LCDClient,
  factoryContract: string,
  tokenContract: string
): Promise<void> {
  process.stdout.write("Test - Query Pair");
  const asset_infos = [
    {
      token: {
        contract_addr: tokenContract,
      },
    },
    {
      native_token: {
        denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4".toString(),
      },
    },
  ];
  const result: any = await terra.wasm.contractQuery(factoryContract, {
    pair: { asset_infos },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryFactoryPairs(
  terra: LocalTerra | LCDClient,
  factoryContract: string
): Promise<void> {
  process.stdout.write("Test - Query Factory Pairs");
  const result: any = await terra.wasm.contractQuery(factoryContract, {
    pairs: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
