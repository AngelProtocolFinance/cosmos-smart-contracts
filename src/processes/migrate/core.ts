/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import {
  LCDClient,
  LocalTerra,
  Msg,
  MsgExecuteContract,
  MsgMigrateContract,
  Wallet,
} from "@terra-money/terra.js";
import {
  sendTransaction,
  storeCode,
  migrateContract,
  migrateContracts,
} from "../../utils/helpers";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateCore(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  indexFund: string,
  cw4GrpApTeam: string,
  // cw4GrpOwners: string,
  cw3ApTeam: string,
  // cw3GuardianAngels: string,
  vaultContracts: string[],
  endowmentContracts: string[]
): Promise<void> {
  // run the migrations desired
  // await migrateRegistrar(terra, apTeam, registrar);
  // await migrateCw4Group(terra, apTeam, cw4GrpApTeam, cw4GrpOwners);
  // await migrateCw3Multisig(terra, apTeam, cw3ApTeam);
  // await migrateGuardianAngelsMultisig(terra, apTeam, cw3GuardianAngels);
  // await migrateIndexFund(terra, apTeam, indexFund);
  // await migrateExistingAccounts(terra, apTeam, registrar, endowmentContracts);
  // await migrateVaults(terra, apTeam, vaultContracts);
}

// -------------------------------------------------
//  Migrate registrar
//--------------------------------------------------
async function migrateRegistrar(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string
): Promise<void> {
  process.stdout.write("Uploading Registrar Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/registrar.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Registrar contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, registrar, codeId, {
    endowtype_fees: {
      endowtype_charity: undefined,
      endowtype_normal: undefined,
    },
    collector_addr: undefined,
    endowments: [],
  });
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate index fund
//--------------------------------------------------
async function migrateIndexFund(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  indexFund: string
): Promise<void> {
  process.stdout.write("Uploading Index Fund Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/index_fund.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Index Fund contract");
  // const result1 = await migrateContract(terra, apTeam, apTeam, indexFund, codeId, {});
  const result1 = await migrateContract(terra, apTeam, apTeam, indexFund, codeId, { });
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate CW4 group
//--------------------------------------------------
async function migrateCw4Group(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw4GrpApTeam: string,
  // cw4GrpOwners: string
): Promise<void> {
  process.stdout.write("Uploading CW4 Group Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/cw4_group.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW4 AP Team Group contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw4GrpApTeam, codeId, {});
  console.log(chalk.green(" Done!"));

  // process.stdout.write("Migrate CW4 Endowment Owners Group contract");
  // const result2 = await migrateContract(terra, apTeam, apTeam, cw4GrpOwners, codeId, {});
  // console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate AP Team multisig
//--------------------------------------------------
async function migrateCw3Multisig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  contract: string
): Promise<void> {
  process.stdout.write("Uploading CW3 MultiSig Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/cw3_multisig.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW3 MultiSig contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, contract, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate vaults
//--------------------------------------------------
async function migrateVaults(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  anchorVaults: string[]
): Promise<void> {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const codeId = await storeCode(
    terra,
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
            await migrateContract(terra, apTeam, apTeam, vault, codeId, {});
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
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  endowmentContracts: string[]
): Promise<void> {
  process.stdout.write("Uploading Accounts Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/accounts.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  // codeId for v1.6 : 58857

  // NOTE: This step is temporarily commented. It should be done through
  // cw3ApTeam contract to update the "accounts_code_id" as 58857 in "registrar"
  /* ---------          ------------- */
  // // Update registrar accounts code ID and migrate all accounts contracts
  process.stdout.write("Update Registrar's Account Code ID stored in configs");
  const result0 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: { accounts_code_id: codeId },
    }),
  ]);
  console.log(chalk.green(" Done!"));
  /* ---------          -------------- */

  process.stdout.write("Migrating existing Endowment Accounts contracts\n");
  let prom = Promise.resolve();
  // eslint-disable-next-line no-async-promise-executor
  prom = prom.then(
    () =>
      new Promise(async (resolve, reject) => {
        try {
          await migrateContracts(terra, apTeam, apTeam, endowmentContracts, codeId, { });
          resolve();
        } catch (e) {
          reject(e);
        }
      })
  );

  await prom;
  console.log(chalk.green(" Done!"));
}
