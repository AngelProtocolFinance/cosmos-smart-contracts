/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import {
  storeCode,
  migrateContract,
} from "../../utils/helpers";
import { testFactoryUpdateConfig } from "../tests/lbp/factory";

// -----------------------------
// Base functions to migrate contracts with 
// -----------------------------
export async function migrateLBPContracts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  pairContract: string,
  routerContract: string,
): Promise<void> {
  // run the migrations desired
  // await migrateFactory(terra, apTeam, factoryContract);
  // await migratePair(terra, apTeam, pairContract, factoryContract);
  await migrateRouter(terra, apTeam, routerContract);
}

// -------------------------------------------------
//  Migrate LBP Factory contract
//--------------------------------------------------
async function migrateFactory(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
): Promise<void> {
  process.stdout.write("Uploading LBP Factory Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/angelprotocol_lbp_factory.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Factory contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, factoryContract, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate LBP Pair contract
//--------------------------------------------------
async function migratePair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
  factoryContract: string,
): Promise<void> {
  process.stdout.write("Uploading LBP Pair wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/angelprotocol_lbp_pair.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Pair contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, pairContract, codeId, {});
  console.log(chalk.green(" Done!"));

  // Update Factory pair_code_id when migrate
  testFactoryUpdateConfig(
    terra,
    apTeam,
    factoryContract,
    undefined,
    undefined,
    codeId,
    pairContract,
    undefined,
    undefined,
    undefined,
  );
}

// -------------------------------------------------
//  Migrate LBP Router contract
//--------------------------------------------------
async function migrateRouter(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerContract: string,
): Promise<void> {
  process.stdout.write("Uploading LBP Router Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/angelprotocol_lbp_router.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate LBP Router contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, routerContract, codeId, {});
  console.log(chalk.green(" Done!"));
}
