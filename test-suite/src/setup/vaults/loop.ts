/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import {
  sendTransaction,
  storeCode,
  instantiateContract,
  getWalletAddress,
  sendMessageViaCw3Proposal,
} from "../../utils/helpers/juno";
import { wasm_path } from "../../utils/config/wasmPaths";
import { localjuno } from "../../utils/config/localjunoConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let chainId: string;
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
  _chainId: string,
  _juno: SigningCosmWasmClient,
  wallets: {
    apTeam: DirectSecp256k1HdWallet;
    apTreasury: DirectSecp256k1HdWallet;
  },
  contracts: {
    registrar: string;
    cw3ApTeam: string;
  },
  config: {
    loopswap_factory: string;
    loopswap_farming: string;
    loopswap_malo_kalo_pair: string;
    loopswap_lp_reward_token: string;
    harvest_to_liquid: string;
    accepted_tokens: any | undefined;
    swapRouter: string;
    nativeToken: any;
  }
): Promise<void> {
  chainId = _chainId;
  juno = _juno;
  apTeam = wallets.apTeam;
  apTreasury = wallets.apTreasury;
  registrar = contracts.registrar;
  cw3ApTeam = contracts.cw3ApTeam;
  apTeamAddr = await getWalletAddress(apTeam);
  apTreasuryAddr = await getWalletAddress(apTreasury);

  await createLoopVaults(
    config.loopswap_factory,
    config.loopswap_farming,
    config.loopswap_malo_kalo_pair,
    config.loopswap_lp_reward_token,
    apTeamAddr,
    apTeamAddr,
    config.harvest_to_liquid,
    config.swapRouter,
    config.nativeToken
  );
}

async function createLoopVaults(
  loopFactory: string,
  loopFarming: string,
  loopPair: string,
  loopStakingRewardToken: string,
  keeper: string,
  tax_collector: string,
  harvest_to_liquid: string,
  swapRouter: string,
  native_token: string
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const vaultCodeId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/loopswap_vault.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // LOOP Vault - #1 (Locked)
  process.stdout.write("Instantiating Vault #1 (Locked) contract");
  const vaultResult1 = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    vaultCodeId,
    {
      acct_type: `locked`, // Locked: 0, Liquid: 1
      sibling_vault: undefined,
      registrar_contract: registrar,
      keeper: keeper,
      tax_collector: tax_collector,
      swap_router: swapRouter,

      lp_factory_contract: loopFactory,
      lp_staking_contract: loopFarming,
      pair_contract: loopPair,
      lp_reward_token: loopStakingRewardToken,
      native_token: native_token,

      reward_to_native_route: [
        {
          loop: {
            offer_asset_info: {
              cw20: localjuno.loopswap.loop_token_contract,
            },
            ask_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
          },
        },
      ],
      native_to_lp0_route: [
        {
          loop: {
            offer_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
            ask_asset_info: {
              cw20: localjuno.loopswap.malo_token_contract,
            },
          },
        },
      ],
      native_to_lp1_route: [
        {
          loop: {
            offer_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
            ask_asset_info: {
              cw20: localjuno.loopswap.kalo_token_contract,
            },
          },
        },
      ],

      name: "Vault Token for MALO-KALO pair",
      symbol: "VTMALOKALO",
      decimals: 6,
    }
  );
  vault1_locked = vaultResult1.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Locked contractAddress")}=${vault1_locked}`
  );

  // Vault - #1 (Liquid)
  process.stdout.write("Instantiating Vault #1 (Liquid) contract");
  const vaultResult2 = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    vaultCodeId,
    {
      acct_type: `liquid`, // Locked: 0, Liquid: 1
      sibling_vault: vault1_locked,
      registrar_contract: registrar,
      keeper: keeper,
      tax_collector: tax_collector,
      swap_router: swapRouter,

      lp_factory_contract: loopFactory,
      lp_staking_contract: loopFarming,
      pair_contract: loopPair,
      lp_reward_token: loopStakingRewardToken,
      native_token: native_token,

      reward_to_native_route: [
        {
          loop: {
            offer_asset_info: {
              cw20: localjuno.loopswap.loop_token_contract,
            },
            ask_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
          },
        },
      ],
      native_to_lp0_route: [
        {
          loop: {
            offer_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
            ask_asset_info: {
              cw20: localjuno.loopswap.malo_token_contract,
            },
          },
        },
      ],
      native_to_lp1_route: [
        {
          loop: {
            offer_asset_info: {
              native: localjuno.networkInfo.nativeToken,
            },
            ask_asset_info: {
              cw20: localjuno.loopswap.kalo_token_contract,
            },
          },
        },
      ],

      name: "Vault Token for MALO-KALO pair",
      symbol: "VTMALOKALO",
      decimals: 6,
    }
  );
  vault1_liquid = vaultResult2.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Liquid contractAddress")}=${vault1_liquid}`
  );

  // Update the "sibling_vault" config of "vault1_locked"
  await sendTransaction(juno, apTeamAddr, vault1_locked, {
    update_config: {
      sibling_vault: vault1_liquid,
      lp_staking_contract: undefined,
      lp_pair_contract: undefined,
      keeper: undefined,
      tax_collector: undefined,

      native_token: undefined,
      reward_to_native_route: undefined,
      native_to_lp0_route: undefined,
      native_to_lp1_route: undefined,
    },
  });

  // Step 3. AP team must add & approve the new vaults in registrar & make #1 the default vault
  process.stdout.write("Add Vault #1 (locked & liquid) in Registrar");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: chainId,
      vault_addr: vault1_locked,
      input_denom: "ujuno",
      yield_token: registrar, // Really needed?
      restricted_from: [],
      acct_type: `locked`,
      vault_type: "native",
    },
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: chainId,
      vault_addr: vault1_liquid,
      input_denom: "ujuno",
      yield_token: registrar, // Really needed?
      restricted_from: [],
      acct_type: `liquid`,
      vault_type: "native",
    },
  });
  console.log(chalk.green(" Done!"));
}
