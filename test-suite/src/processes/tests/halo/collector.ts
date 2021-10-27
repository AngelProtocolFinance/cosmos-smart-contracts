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
  pleb: Wallet,
  govContract: string,
  collectorContract: string,
  reward_factor: string
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update collector config");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        collectorContract,
        {
          update_config: { reward_factor },
        },
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only owner cannot update collector config");

  await expect(
    sendTransaction(terra, apTeam, [ // replace apTeam to gov contract (Wallet)
      new MsgExecuteContract(
        govContract,
        collectorContract,
        {
          update_config: { reward_factor },
        },
      ),
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
  collectorContract: string,
): Promise<void> {
  process.stdout.write("Test - Anyone can swap asset token => HALO token");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        collectorContract,
        {
          sweep: { denom: "uusd" },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
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
