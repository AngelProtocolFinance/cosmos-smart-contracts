/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction, storeCode, migrateContract, storeAndMigrateContract } from "../../utils/helpers";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateCore(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  indexFund: string,
  accounts: string,
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  cw3ReviewTeam: string,
  vaultContracts: string[],
): Promise<void> {
  // run the migrations desired
  // await storeAndMigrateContract(juno, apTeam, registrar, 'registrar.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw4GrpApTeam, 'cw4_group.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ApTeam, 'cw3_apteam.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ReviewTeam, 'cw3_applications.wasm');
  // await storeAndMigrateContract(juno, apTeam, indexFund, 'index_fund.wasm');
  // await storeAndMigrateContract(juno, apTeam, accounts, 'accounts.wasm');
  // await migrateVaults(juno, apTeam, vaultContracts);
}

// -------------------------------------------------
//  Migrate vaults
//--------------------------------------------------
async function migrateVaults(
  juno: SigningCosmWasmClient,
  apTeam: string,
  vaults: string[]
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/loopswap_vault.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  vaults.forEach((vault) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await migrateContract(juno, apTeam, vault, codeId, {});
            console.log(chalk.green(`Vault ${id++} of ${vaults.length} - Done!`));
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });

  await prom;
  console.log(chalk.green(" Done!"));
}
