/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update collector config
//
// SCENARIO:
// Pleb cannot update contract config, only gov contract can update config
//
//----------------------------------------------------------------------------------------
export async function testCollectorUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  collectorContract: string,
  reward_factor: string | undefined,
  gov_contract: string | undefined,
  swap_factory: string | undefined
): Promise<void> {
  process.stdout.write("Test - Gov contract update collector config");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, collectorContract, {
        update_config: {
          reward_factor,
          gov_contract,
          swap_factory,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Sweep
//
// SCENARIO:
// Anyone can execute sweep function to swap
// asset token => HALO token and distribute
// result HALO token to gov contract
//
//----------------------------------------------------------------------------------------
export async function testCollectorSweep(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  collectorContract: string
): Promise<void> {
  process.stdout.write("Test - Anyone can sweep asset token => HALO token");

  let result = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, collectorContract, {
      sweep: { denom: "uusd" },
    }),
  ]);

  let distribution_amount = result.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "distribute_amount";
    })?.value as string;

  console.log(
    `Distributed to Gov Stakers: ${distribution_amount}`,
    chalk.green(" Passed!")
  );
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryCollectorConfig(
  terra: LocalTerra | LCDClient,
  collectorContract: string
): Promise<void> {
  process.stdout.write("Test - Query Collector Config");
  const result: any = await terra.wasm.contractQuery(collectorContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryCollectorPair(
  terra: LocalTerra | LCDClient,
  collectorContract: string
): Promise<void> {
  process.stdout.write("Test - Query Collector pair");
  const result: any = await terra.wasm.contractQuery(collectorContract, {
    pair: { denom: "uusd" },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
