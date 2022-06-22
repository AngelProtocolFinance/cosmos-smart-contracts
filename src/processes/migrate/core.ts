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
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  vaultContracts: string[],
  endowmentContracts: string[]
): Promise<void> {
  // run the migrations desired
  // await storeAndMigrateContract(juno, apTeam, registrar, 'registrar.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw4GrpApTeam, 'cw4_group.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ApTeam, 'cw3_multisig.wasm');
  // await storeAndMigrateContract(juno, apTeam, indexFund, 'index_fund.wasm');
  // await migrateVaults(juno, apTeam, vaultContracts);
  // await migrateExistingAccounts(juno, apTeam, registrar, endowmentContracts);
}

// -------------------------------------------------
//  Migrate vaults
//--------------------------------------------------
async function migrateVaults(
  juno: SigningCosmWasmClient,
  apTeam: string,
  anchorVaults: string[]
): Promise<void> {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/anchor.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  anchorVaults.forEach((vault) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await migrateContract(juno, apTeam, vault, codeId, {});
            console.log(chalk.green(`anchorVault #${id++} - Done!`));
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

// -------------------------------------------------
//  Migrate a list of existing Endowment contracts
//--------------------------------------------------
async function migrateExistingAccounts(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  accounts_wasm: number | undefined,
  endowmentContracts: any[] // [ [ endow_address, migrate_msg ], ... ]
): Promise<void> {
  let codeId: number;
  if (!accounts_wasm) {
    process.stdout.write("Uploading Accounts Wasm");
    codeId = await storeCode(
      juno,
      apTeam,
      path.resolve(__dirname, `${wasm_path.core}/accounts.wasm`)
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

    // Update registrar accounts code ID value
    process.stdout.write("Update Registrar's Account Code ID stored in configs");
    const result0 = await sendTransaction(juno, apTeam, registrar, {
      update_config: { accounts_code_id: codeId },
    });
    console.log(chalk.green(" Done!"));
  } else {
    codeId = accounts_wasm;
  }

  // migrate all accounts contracts
  process.stdout.write("Migrating existing Endowment Accounts contracts\n");
  let prom = Promise.resolve();
  endowmentContracts.forEach((endow) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await migrateContract(juno, apTeam, endow[0], codeId, endow[1]);
            console.log(chalk.green(`Endowment ${endow[0]}`));
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });

  await prom;
  console.log(chalk.green(`${endowmentContracts.length} endowments migrated!`));
}
