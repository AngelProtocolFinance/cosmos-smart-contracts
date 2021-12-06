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
  factory_contract: string,
  owner: string | undefined,
  token_code_id: number | undefined,
  pair_code_id: number | undefined,
  pair_contract: string,
  collector_addr: string | undefined,
  commission_rate: string | undefined,
): Promise<void> {
  process.stdout.write("Test - Only owner update factory config");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factory_contract,
        {
          update_config: { owner, token_code_id, pair_code_id, pair_contract, collector_addr, commission_rate },
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
  factory_contract: string
): Promise<void> {
  process.stdout.write("Test - Query Factory Config");
  const result: any = await terra.wasm.contractQuery(factory_contract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPair(
  terra: LocalTerra | LCDClient,
  factory_contract: string,
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

  const result: any = await terra.wasm.contractQuery(factory_contract, {
    pair: { asset_infos },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryPairs(
  terra: LocalTerra | LCDClient,
  factory_contract: string,
): Promise<void> {
  process.stdout.write("Test - Query Pairs");

  const result: any = await terra.wasm.contractQuery(factory_contract, {
    pairs: { start_after: undefined, limit: undefined },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
