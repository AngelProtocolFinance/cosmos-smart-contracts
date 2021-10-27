/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update distributor config
//
// SCENARIO:
// Pleb cannot update contract config, only owner can update config
//
//----------------------------------------------------------------------------------------
export async function testDistributorUpdateConfig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pleb: Wallet,
  distributorContract: string,
  spend_limit: string,
): Promise<void> {
  process.stdout.write("Test - Pleb cannot update distributor config");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        distributorContract,
        {
          update_config: { spend_limit },
        },
      ),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write("Test - Only gov contract can update distributor config");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        distributorContract,
        {
          update_config: { spend_limit },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Spend
//
// SCENARIO:
// Owner can execute spend operation to send
// `amount` of HALO token to `recipient` for community purpose
//
//----------------------------------------------------------------------------------------
export async function testDistributorSpend(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  distributorContract: string,
  receipient: string,
  amount: string
): Promise<void> {
  process.stdout.write("Test - Send `amount` of HALO token to `receipient` for community purpose");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        distributorContract,
        {
          spend: { receipient, amount },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Add distributor
//
// SCENARIO:
// Gov contract can add new distributor
//
//----------------------------------------------------------------------------------------
export async function testDistributorAdd(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  distributorContract: string,
  distributor: string
): Promise<void> {
  process.stdout.write("Test - Only gov contract can add new distributor");

  await expect(
    sendTransaction(terra, apTeam, [ // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(
        govContract,
        distributorContract,
        {
          add_distributor: { distributor },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Remove distributor
//
// SCENARIO:
// Gov contract can remove distributor
//
//----------------------------------------------------------------------------------------
export async function testDistributorRemove(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  govContract: string,
  distributorContract: string,
  distributor: string
): Promise<void> {
  process.stdout.write("Test - Only gov contract can remove new distributor");

  await expect(
    sendTransaction(terra, apTeam, [ // TODO: replace apTeam to govContract(Wallet)
      new MsgExecuteContract(
        govContract,
        distributorContract,
        {
          remove_distributor: { distributor },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryDistributorConfig(
  terra: LocalTerra | LCDClient,
  distributorContract: string
): Promise<void> {
  process.stdout.write("Test - Query Distributor Config");
  const result: any = await terra.wasm.contractQuery(distributorContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
