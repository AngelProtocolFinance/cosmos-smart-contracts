/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
  storeCode,
  migrateContract,
} from "../../utils/helpers";

// -----------------------------
// Base functions to migrate contracts with 
// -----------------------------
export async function migrateContracts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  indexFund: string,
  cw4GrpApTeam: string,
  cw4GrpOwners: string,
  cw3ApTeam: string,
  cw3GuardianAngels: string,
  anchorVaults: string[],
  endowmentContracts: string[]
): Promise<void> {
  // run the migrations desired
  await migrateRegistrar(terra, apTeam, registrar);
  await migrateCw4Group(terra, apTeam, cw4GrpApTeam, cw4GrpOwners);
  await migrateApTeamMultisig(terra, apTeam, cw3ApTeam);
  await migrateGuardianAngelsMultisig(terra, apTeam, cw3GuardianAngels);
  await migrateIndexFund(terra, apTeam, indexFund);
  await migrateAccounts(terra, apTeam, registrar, endowmentContracts);
  await migrateVaults(terra, apTeam, anchorVaults);
}

// -------------------------------------------------
//  Migrate registrar
//--------------------------------------------------
async function migrateRegistrar(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
): Promise<void> {
  process.stdout.write("Uploading Registrar Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/registrar.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  process.stdout.write("Migrate Registrar contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, registrar, codeId, {});
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
    path.resolve(__dirname, "../../../../artifacts/index_fund.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  process.stdout.write("Migrate Index Fund contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, indexFund, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate CW4 group
//--------------------------------------------------
async function migrateCw4Group(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw4GrpApTeam: string,
  cw4GrpOwners: string
): Promise<void> {
  process.stdout.write("Uploading CW4 Group Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/cw4_group.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW4 AP Team Group contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw4GrpApTeam, codeId, {});
  console.log(chalk.green(" Done!"));

  process.stdout.write("Migrate CW4 Endowment Owners Group contract");
  const result2 = await migrateContract(terra, apTeam, apTeam, cw4GrpOwners, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate AP Team multisig
//--------------------------------------------------
async function migrateApTeamMultisig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw3ApTeam: string
): Promise<void> {
  process.stdout.write("Uploading AP Team MultiSig Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/ap_team_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate AP Team MultiSig contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw3ApTeam, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate guardian angels multisig
//--------------------------------------------------
async function migrateGuardianAngelsMultisig(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw3GuardianAngels: string
): Promise<void> {
  process.stdout.write("Uploading Guardian Angels MultiSig Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/guardian_angels_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Guardian Angels MultiSig contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw3GuardianAngels, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate vaults
//--------------------------------------------------
async function migrateVaults(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  anchorVaults:  string[],
): Promise<void> {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/anchor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  anchorVaults.forEach(vault => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(() => new Promise(async (resolve, reject) => {
      try {
        await migrateContract(terra, apTeam, apTeam, vault, codeId, {});
        console.log(chalk.green(`anchorVault #${id ++} - Done!`));
        resolve();
      } catch(e) {
        reject(e);
      }
    }));
  });

  await prom;
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate endowments
//--------------------------------------------------
async function migrateAccounts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  endowmentContracts: string[]
): Promise<void> {
  process.stdout.write("Uploading Accounts Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/accounts.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  // Update registrar accounts code ID and migrate all accounts contracts
  process.stdout.write("Update Registrar's Account Code ID stored in configs");
  const result0 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: { accounts_code_id: codeId }
    }),
  ]);
  console.log(chalk.green(" Done!"));
  
  process.stdout.write("Migrate Accounts contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  endowmentContracts.forEach(endowment => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(() => new Promise(async (resolve, reject) => {
      try {
        await migrateContract(terra, apTeam, apTeam, endowment, codeId, {});
        console.log(chalk.green(`#${id ++} - Done!`));
        resolve();
      } catch(e) {
        reject(e);
      }
    }));
  });

  await prom;
  console.log(chalk.green(" Done!"));
}
