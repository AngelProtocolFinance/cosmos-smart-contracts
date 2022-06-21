/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

chai.use(chaiAsPromised);

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
  juno: SigningCosmWasmClient,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Config");
  const result: any = await juno.wasm.contractQuery(vault, {
    vault_config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
