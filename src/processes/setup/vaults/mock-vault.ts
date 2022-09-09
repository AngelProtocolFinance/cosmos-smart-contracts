/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { sendTransaction, storeCode, instantiateContract, getWalletAddress, sendMessageViaCw3Proposal } from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTreasury: DirectSecp256k1HdWallet;

// wallet addresses
let apTeamAddr: string;
let apTreasuryAddr: string;

// contracts
let registrar: string;
let cw3ApTeam: string;
let vault1_locked: string;
let vault1_liquid: string;
let vault2_locked: string;
let vault2_liquid: string;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------

export async function setupMockVaults(
  _juno: SigningCosmWasmClient,
  wallets: {
    apTeam: DirectSecp256k1HdWallet;
    apTreasury: DirectSecp256k1HdWallet;
  },
  contracts: {
  	registrar: string,
  	cw3ApTeam: string,
  },
  config: {
    harvest_to_liquid: string;
    tax_per_block: string;
    accepted_tokens: any | undefined;
  }
): Promise<void> {
  juno = _juno;
  apTeam = wallets.apTeam;
  apTreasury = wallets.apTreasury;
  registrar = contracts.registrar;
  cw3ApTeam = contracts.cw3ApTeam;
  apTeamAddr = await getWalletAddress(apTeam);
  apTreasuryAddr = await getWalletAddress(apTreasury);

  await createMockVaults(config.harvest_to_liquid, config.tax_per_block, config.accepted_tokens);
}

async function createMockVaults(
  harvest_to_liquid: string,
  tax_per_block: string,
  accepted_tokens: any,
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const vaultCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.mock_vault}/mock_vault.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // Vault - #1
  process.stdout.write("Instantiating Vault (#1) locked & liquid contracts");
  const vaultLockedResult1 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    acct_type: "locked",
    input_denom: "ujuno", // testnet placeholder
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #1 (locked)",
    symbol: "apV1Lk",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault1_locked = vaultLockedResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Locked contractAddress")}=${vault1_locked}`);

  const vaultLiquidResult1 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    acct_type: "liquid",
    input_denom: "ujuno", // testnet placeholder
    yield_token: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #1 (liquid)",
    symbol: "apV1Lq",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault1_liquid = vaultLiquidResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Liquid contractAddress")}=${vault1_liquid}`);

  // Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Vault (#2) locked & liquid contracts");
  const vaultLockedResult2 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    acct_type: "locked",
    input_denom: "ujuno", // testnet placeholder
    yield_token: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #2 (locked)",
    symbol: "apV2Lk",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault2_locked = vaultLockedResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Locked contractAddress")}=${vault2_locked}`);

  const vaultLiquidResult2 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    acct_type: "liquid",
    input_denom: "ujuno", // testnet placeholder
    yield_token: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #2 (liquid)",
    symbol: "apV2Lq",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault2_liquid = vaultLiquidResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Liquid contractAddress")}=${vault2_liquid}`);

  // Step 3. AP team must add & approve the new vaults in registrar & make #1 the default vault
  process.stdout.write("Add Vaults into Registrar");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault1_locked,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `locked`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault1_liquid,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `liquid`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault2_locked,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `locked`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault2_liquid,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `liquid`,
    }
  });
  console.log(chalk.green(" Done!"));
}
