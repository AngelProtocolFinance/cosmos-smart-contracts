/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra } from "@terra-money/terra.js";

chai.use(chaiAsPromised);

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
  terra: LocalTerra | LCDClient,
  anchorVault1: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Config");
  const result: any = await terra.wasm.contractQuery(anchorVault1, {
    vault_config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
