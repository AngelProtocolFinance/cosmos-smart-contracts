/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  storeCode,
  migrateContract,
  sendTransaction,
} from "../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;
// -----------------------------
// Base functions to migrate contracts with 
// -----------------------------
export async function migrateAMMContracts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  pairContract: string,
  routerContract: string,
  commission_rate: string,
): Promise<void> {
  // run the migrations desired
  await migrateFactory(terra, apTeam, factoryContract);
  await migratePair(terra, apTeam, pairContract);
  await migrateRouter(terra, apTeam, routerContract);

  // Update commission rate post-LBP
  await updateCommissionRate(
    terra,
    apTeam,
    factoryContract,
    undefined,
    undefined,
    undefined,
    pairContract,
    undefined,
    commission_rate,
  );
}

// -------------------------------------------------
//  Migrate Factory contract
//--------------------------------------------------
async function migrateFactory(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
): Promise<void> {
  process.stdout.write("Uploading Factory wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/amm_factory.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Factory contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, factoryContract, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate Pair contract
//--------------------------------------------------
async function migratePair(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  pairContract: string,
): Promise<void> {
  process.stdout.write("Uploading Pair wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/amm_pair.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Pair contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, pairContract, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate Router contract
//--------------------------------------------------
async function migrateRouter(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  routerContract: string,
): Promise<void> {
  process.stdout.write("Uploading Router wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../../../artifacts/amm_router.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Router contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, routerContract, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Update commission rate post-LBP
//--------------------------------------------------
async function updateCommissionRate(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  factoryContract: string,
  owner: string | undefined,
  token_code_id: number | undefined,
  pair_code_id: number | undefined,
  pair_contract: string,
  collector_addr: string | undefined,
  commission_rate: string
): Promise<void> {
  process.stdout.write("Update commission rate post-LBP");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(
        apTeam.key.accAddress,
        factoryContract,
        {
          update_config: { owner, token_code_id, pair_code_id, pair_contract, collector_addr, commission_rate },
        },
      ),
    ])
  );
  console.log(chalk.green(" Passed!"));
}
