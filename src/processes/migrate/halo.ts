/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { storeCode, migrateContract } from "../../utils/helpers";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateHalo(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloGovHodler: string,
  haloStaking: string,
  haloVesting: string
): Promise<void> {
  // run the migrations desired
  // await migrateHaloAirdrop(juno, apTeam, haloAirdrop);
  // await migrateHaloCollector(juno, apTeam, haloCollector);
  // await migrateHaloCommunity(juno, apTeam, haloCommunity);
  // await migrateHaloDistributor(juno, apTeam, haloDistributor);
  // await migrateHaloGov(juno, apTeam, haloGov);
  // await migrateHaloGovHodler(juno, apTeam, haloGovHodler);
  // await migrateHaloStaking(juno, apTeam, haloStaking);
  // await migrateHaloVesting(juno, apTeam, haloVesting);
}

// -------------------------------------------------
//  Migrate HALO airdrop
//--------------------------------------------------
async function migrateHaloAirdrop(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloAirdrop: string
): Promise<void> {
  process.stdout.write("Uploading HALO airdrop Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_airdrop.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO airdrop contract");
  const result1 = await migrateContract(juno, apTeam, haloAirdrop, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO collector
//--------------------------------------------------
async function migrateHaloCollector(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloCollector: string
): Promise<void> {
  process.stdout.write("Uploading HALO collector");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_collector.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO collector contract");
  const result1 = await migrateContract(juno, apTeam, haloCollector, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO community
//--------------------------------------------------
async function migrateHaloCommunity(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloCommunity: string
): Promise<void> {
  process.stdout.write("Uploading HALO community Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_community.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO community contract");
  const result1 = await migrateContract(juno, apTeam, haloCommunity, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO distributor
//--------------------------------------------------
async function migrateHaloDistributor(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloDistributor: string
): Promise<void> {
  process.stdout.write("Uploading HALO distributor Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_distributor.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO distributor contract");
  const result1 = await migrateContract(juno, apTeam, haloDistributor, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO gov
//--------------------------------------------------
async function migrateHaloGov(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloGov: string
): Promise<void> {
  process.stdout.write("Uploading HALO gov Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_gov.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO gov contract");
  const result1 = await migrateContract(juno, apTeam, haloGov, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO gov hodler
//--------------------------------------------------
async function migrateHaloGovHodler(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloGovHodler: string
): Promise<void> {
  process.stdout.write("Uploading HALO gov hodler Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_gov_hodler.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO gov hodler contract");
  const result1 = await migrateContract(juno, apTeam, haloGovHodler, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO staking
//--------------------------------------------------
async function migrateHaloStaking(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloStaking: string
): Promise<void> {
  process.stdout.write("Uploading HALO staking Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_staking.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO staking contract");
  const result1 = await migrateContract(juno, apTeam, haloStaking, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO vesting
//--------------------------------------------------
async function migrateHaloVesting(
  juno: SigningCosmWasmClient,
  apTeam: string,
  haloVesting: string
): Promise<void> {
  process.stdout.write("Uploading HALO vesting Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_vesting.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO vesting contract");
  const result1 = await migrateContract(juno, apTeam, haloVesting, codeId, {});
  console.log(chalk.green(" Done!"));
}
