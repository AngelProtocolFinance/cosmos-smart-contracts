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
  owner: string | undefined,
  token_code_id: number | undefined,
  pair_code_id: number | undefined,
  commission_rate: string | undefined,
  collector_addr: string | undefined,
): Promise<void> {
  process.stdout.write("Test - Only owner can update Factory config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
          update_config: {
            owner,
            token_code_id,
            pair_code_id,
            commission_rate,
            collector_addr,
          },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Update Pair config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testFactoryUpdatePair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  pair: string,
  end_time: number | undefined,
): Promise<void> {
  process.stdout.write("Test - Only owner can update Factory config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
          update_pair: {
            pair,
            end_time,
          },
        },
      ),
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
  start_time: number,
  end_time: number | undefined,
  description: string | undefined,
): Promise<void> {
  process.stdout.write("Test - Only the owner can execute it to create swap pair");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
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
            start_time,
            end_time,
            description,
          }
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
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
): Promise<void> {
  process.stdout.write("Test - Factory Pair owner can remote from list of pairs");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
          unregister: {
            asset_infos: [
              {
                token: {
                  contract_addr: tokenContract,
                }
              },
              {
                native_token: {
                  denom: "uusd".toString()
                }
              }
            ]
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
  tokenContract: string,
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
        denom: "uusd".toString()
      }
    }
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
