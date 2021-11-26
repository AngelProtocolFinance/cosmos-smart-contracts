/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update factory config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testFactoryUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pleb: Wallet,
  factoryContract: string,
  owner: string | undefined,
  token_code_id: number | undefined,
  pair_code_id: number | undefined,
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update factory config");
  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        factoryContract,
        {
          update_config: { owner, token_code_id: pair_code_id },
        },
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only owner update factory config");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
          update_config: { owner, token_code_id, pair_code_id },
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

export async function testQueryPair(
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
  ]

  const result: any = await terra.wasm.contractQuery(factoryContract, {
    pair: { asset_infos },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairs(
  terra: LocalTerra | LCDClient,
  factoryContract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pairs");

  const result: any = await terra.wasm.contractQuery(factoryContract, {
    pairs: { start_after: undefined, limit: undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
