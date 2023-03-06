/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { storeAndMigrateContract } from "../../utils/juno/helpers";
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
    // await storeAndMigrateContract(juno, apTeam, haloAirdrop, 'halo_airdrop.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloCollector, 'halo_collector.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloCommunity, 'halo_community.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloDistributor, 'halo_distributor.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloGov, 'halo_gov.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloGovHodler, 'halo_gov_hodler.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloStaking, 'halo_staking.wasm');
    // await storeAndMigrateContract(juno, apTeam, haloVesting, 'halo_vesting.wasm');
}
