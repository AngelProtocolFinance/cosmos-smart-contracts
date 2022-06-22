/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  distributorContract: string,
  spend_limit: string | undefined,
  gov_contract: string | undefined
): Promise<void> {
  process.stdout.write("Test - Only gov contract can update distributor config");

  await expect(
    sendTransaction(juno, apTeam, distributorContract, {
      update_config: { spend_limit, gov_contract },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  distributorContract: string,
  receipient: string,
  amount: string
): Promise<void> {
  process.stdout.write(
    "Test - Send `amount` of HALO token to `receipient` for community purpose"
  );

  await expect(
    sendTransaction(juno, apTeam, distributorContract, {
      spend: { receipient, amount },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  distributorContract: string,
  distributor: string
): Promise<void> {
  process.stdout.write("Test - Only gov contract can add new distributor");

  await expect(
    sendTransaction(juno, govContract, distributorContract, {
      add_distributor: { distributor },
    })
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  govContract: string,
  distributorContract: string,
  distributor: string
): Promise<void> {
  process.stdout.write("Test - Only gov contract can remove new distributor");

  await expect(
    sendTransaction(juno, govContract, distributorContract, {
      remove_distributor: { distributor },
    })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryDistributorConfig(
  juno: SigningCosmWasmClient,
  distributorContract: string
): Promise<void> {
  process.stdout.write("Test - Query Distributor Config");
  const result: any = await juno.queryContractSmart(distributorContract, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
