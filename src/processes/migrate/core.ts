/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction, storeCode, migrateContract } from "../../utils/helpers";
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
  // await migrateRegistrar(juno, apTeam, registrar);
  // await migrateCw4Group(juno, apTeam, cw4GrpApTeam);
  // await migrateCw3Multisig(juno, apTeam, cw3ApTeam);
  // await migrateIndexFund(juno, apTeam, indexFund);
  // await migrateExistingAccounts(juno, apTeam, registrar, endowmentContracts);
  // await migrateVaults(juno, apTeam, vaultContracts);
}

// -------------------------------------------------
//  Migrate registrar
//--------------------------------------------------
async function migrateRegistrar(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string
): Promise<void> {
  process.stdout.write("Uploading Registrar Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/registrar.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Registrar contract");
  const result1 = await migrateContract(juno, apTeam, registrar, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate index fund
//--------------------------------------------------
async function migrateIndexFund(
  juno: SigningCosmWasmClient,
  apTeam: string,
  indexFund: string
): Promise<void> {
  process.stdout.write("Uploading Index Fund Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/index_fund.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Index Fund contract");
  const result1 = await migrateContract(juno, apTeam, indexFund, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate CW4 group
//--------------------------------------------------
async function migrateCw4Group(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw4Grp: string
): Promise<void> {
  process.stdout.write("Uploading CW4 Group Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/cw4_group.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW4 Group contract");
  const result1 = await migrateContract(juno, apTeam, cw4Grp, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate AP Team multisig
//--------------------------------------------------
async function migrateCw3Multisig(
  juno: SigningCosmWasmClient,
  apTeam: string,
  contract: string
): Promise<void> {
  process.stdout.write("Uploading CW3 MultiSig Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/ap_team_multisig.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW3 MultiSig contract");
  const result1 = await migrateContract(juno, apTeam, contract, codeId, {});
  console.log(chalk.green(" Done!"));
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
