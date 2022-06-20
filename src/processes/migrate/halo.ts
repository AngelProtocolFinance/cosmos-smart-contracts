/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LcdClient,  Wallet } from "@cosmjs/launchpad";
import { storeCode, migrateContract } from "../../utils/helpers";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateHalo(
  juno: LcdClient,
  apTeam: Wallet,
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
  juno: LcdClient,
  apTeam: Wallet,
  haloAirdrop: string
): Promise<void> {
  process.stdout.write("Uploading HALO airdrop Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_airdrop.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO airdrop contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloAirdrop, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO collector
//--------------------------------------------------
async function migrateHaloCollector(
  juno: LcdClient,
  apTeam: Wallet,
  haloCollector: string
): Promise<void> {
  process.stdout.write("Uploading HALO collector");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_collector.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO collector contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloCollector, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO community
//--------------------------------------------------
async function migrateHaloCommunity(
  juno: LcdClient,
  apTeam: Wallet,
  haloCommunity: string
): Promise<void> {
  process.stdout.write("Uploading HALO community Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_community.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO community contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloCommunity, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO distributor
//--------------------------------------------------
async function migrateHaloDistributor(
  juno: LcdClient,
  apTeam: Wallet,
  haloDistributor: string
): Promise<void> {
  process.stdout.write("Uploading HALO distributor Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_distributor.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO distributor contract");
  const result1 = await migrateContract(
    terra,
    apTeam,
    apTeam,
    haloDistributor,
    codeId,
    {}
  );
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO gov
//--------------------------------------------------
async function migrateHaloGov(
  juno: LcdClient,
  apTeam: Wallet,
  haloGov: string
): Promise<void> {
  process.stdout.write("Uploading HALO gov Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_gov.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO gov contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloGov, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO gov hodler
//--------------------------------------------------
async function migrateHaloGovHodler(
  juno: LcdClient,
  apTeam: Wallet,
  haloGovHodler: string
): Promise<void> {
  process.stdout.write("Uploading HALO gov hodler Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_gov_hodler.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO gov hodler contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloGovHodler, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO staking
//--------------------------------------------------
async function migrateHaloStaking(
  juno: LcdClient,
  apTeam: Wallet,
  haloStaking: string
): Promise<void> {
  process.stdout.write("Uploading HALO staking Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_staking.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO staking contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloStaking, codeId, {});
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate HALO vesting
//--------------------------------------------------
async function migrateHaloVesting(
  juno: LcdClient,
  apTeam: Wallet,
  haloVesting: string
): Promise<void> {
  process.stdout.write("Uploading HALO vesting Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/halo_vesting.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate HALO vesting contract");
  const result1 = await migrateContract(juno, apTeam, apTeam, haloVesting, codeId, {});
  console.log(chalk.green(" Done!"));
}
