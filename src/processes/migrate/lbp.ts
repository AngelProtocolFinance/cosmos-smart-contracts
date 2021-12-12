/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import { storeCode, migrateContract } from "../../utils/helpers";
import { testFactoryUpdateConfig } from "../tests/lbp/factory";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateLbp(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  pairContract: string,
  routerContract: string
): Promise<void> {
  // run the migrations desired
  await migrateFactory(terra, apTeam, factoryContract);
  const pairCodeId = await migratePair(terra, apTeam, pairContract);
  await migrateRouter(terra, apTeam, routerContract);

  // Update Factory pair_code_id when migrate
  await testFactoryUpdateConfig(
    terra,
    apTeam,
    factoryContract,
    undefined,
    undefined,
    pairCodeId,
    undefined,
    undefined,
    undefined
  );
}

// -------------------------------------------------
//  Migrate LBP Factory contract
//--------------------------------------------------
async function migrateFactory(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string
): Promise<void> {
  process.stdout.write("Uploading LBP Factory Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.lbp}/astroport_lbp_factory.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Factory contract");
  const result1 = await migrateContract(
    terra,
    apTeam,
    apTeam,
    factoryContract,
    codeId,
    {}
  );
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate LBP Pair contract
//--------------------------------------------------
async function migratePair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string
): Promise<number> {
  process.stdout.write("Uploading LBP Pair wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.lbp}/astroport_lbp_pair.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Pair contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, pairContract, codeId, {});
  console.log(chalk.green(" Done!"));

  return codeId;
}

// -------------------------------------------------
//  Migrate LBP Router contract
//--------------------------------------------------
async function migrateRouter(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerContract: string
): Promise<void> {
  process.stdout.write("Uploading LBP Router Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.lbp}/astroport_lbp_router.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Router contract");
  const result1 = await migrateContract(
    terra,
    apTeam,
    apTeam,
    routerContract,
    codeId,
    {}
  );
  console.log(chalk.green(" Done!"));
}
