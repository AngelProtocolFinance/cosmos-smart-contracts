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

export async function setupLoopVaults(
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
    loopswap_factory: string;
    loopswap_farming: string;
    loopswap_loop_juno_pair: string;
    loopswap_lp_reward_token: string;
    harvest_to_liquid: string;
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

  await createLoopVaults(config.loopswap_factory, config.loopswap_farming, config.loopswap_loop_juno_pair, config.loopswap_lp_reward_token, apTeamAddr, apTeamAddr, config.harvest_to_liquid);
}

async function createLoopVaults(
  loopFactory: string,
  loopFarming: string,
  loopPair: string,
  loopStakingRewardToken: string,
  keeper: string,
  tax_collector: string,
  harvest_to_liquid: string,
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const vaultCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/loopswap_vault.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // LOOP Vault - #1 (Locked)
  process.stdout.write("Instantiating Vault #1 (Locked) contract");
  const vaultResult1 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    acct_type: `locked`, // Locked: 0, Liquid: 1
    sibling_vault: undefined,
    registrar_contract: registrar,
    keeper: keeper,
    tax_collector: tax_collector,
    lp_factory_contract: loopFactory,
    lp_staking_contract: loopFarming,
    pair_contract: loopPair,
    lp_reward_token: loopStakingRewardToken,
    name: "Vault Token for LOOP-JUNO pair",
    symbol: "VTLOOPJUNO",
    decimals: 6,

    harvest_to_liquid: harvest_to_liquid,
  });
  vault1_locked = vaultResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Liquid contractAddress")}=${vault1_locked}`);

  // Vault - #1 (Liquid)
  process.stdout.write("Instantiating Vault #1 (Liquid) contract");
  const vaultResult2 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    acct_type: `liquid`, // Locked: 0, Liquid: 1
    sibling_vault: vault1_locked,
    registrar_contract: registrar,
    keeper: keeper,
    tax_collector: tax_collector,
    lp_factory_contract: loopFactory,
    lp_staking_contract: loopFarming,
    pair_contract: loopPair,
    lp_reward_token: loopStakingRewardToken,
    name: "Vault Token for LOOP-JUNO pair",
    symbol: "VTLOOPJUNO",
    decimals: 6,

    harvest_to_liquid: harvest_to_liquid,
  });
  vault1_liquid = vaultResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("Liquid contractAddress")}=${vault1_liquid}`);

  // Update the "sibling_vault" config of "vault1_locked"
  await sendTransaction(juno, apTeamAddr, vault1_locked, {
    update_config: {
      sibling_vault: vault1_liquid,
      lp_staking_contract: undefined,
      lp_pair_contract: undefined,
      keeper: undefined,
      tax_collector: undefined,
    }
  });

  // Step 3. AP team must add & approve the new vaults in registrar & make #1 the default vault
  process.stdout.write("Add Vault #1 (locked & liquid) in Registrar");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: undefined,
      vault_addr: vault1_locked,
      input_denom: "ujuno",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `locked`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: undefined,
      vault_addr: vault1_liquid,
      input_denom: "ujuno",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `liquid`,
    }
  });
  console.log(chalk.green(" Done!"));
}
