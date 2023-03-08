/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import {
  sendTransaction,
  storeCode,
  instantiateContract,
  storeAndInstantiateContract,
  getWalletAddress,
  sendMessageViaCw3Proposal,
  sendApplicationViaCw3Proposal,
} from "../../../utils/juno/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { localjuno } from "../../../config/localjunoConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let networkUrl: string;

let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeam2: DirectSecp256k1HdWallet;
let apTreasury: DirectSecp256k1HdWallet;

// wallet addresses
let apTeamAddr: string;
let apTeam2Addr: string;
let apTreasuryAddr: string;

// wasm codes
let cw4Group: number;
let cw3MultiSigEndowment: number;

// contract addresses
let registrar: string;
let accounts: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let donationMatchCharities: string;
let swapRouter: string;
let settingsController: string;

let vault1: string;
let vault2: string;

// endowment IDs
let endow_1_id: number;
let endow_2_id: number;
let endow_3_id: number;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------

export async function setupCore(
  _networkUrl: any,
  _juno: SigningCosmWasmClient,
  wallets: {
    apTeam: DirectSecp256k1HdWallet;
    apTeam2: DirectSecp256k1HdWallet;
    apTeam3: DirectSecp256k1HdWallet;
    apTreasury: DirectSecp256k1HdWallet;
  },
  config: {
    tax_rate: string;
    threshold_absolute_percentage: string;
    max_voting_period_height: number;
    fund_rotation: number | undefined;
    harvest_to_liquid: string;
    funding_goal: string | undefined;
    fund_member_limit: number | undefined;
    charity_cw3_threshold_abs_perc: string;
    charity_cw3_max_voting_period: number;
    accepted_tokens: any | undefined;
    axelar_gateway: string;
    axelar_ibc_channel: string;
  }
): Promise<void> {
  networkUrl = _networkUrl;
  juno = _juno;
  apTeam = wallets.apTeam;
  apTeam2 = wallets.apTeam2;
  apTreasury = wallets.apTreasury;

  apTeamAddr = await getWalletAddress(apTeam);
  apTeam2Addr = await getWalletAddress(apTeam2);
  apTreasuryAddr = await getWalletAddress(apTreasury);

  await setup(
    config.tax_rate,
    apTreasuryAddr,
    config.threshold_absolute_percentage,
    config.max_voting_period_height,
    config.fund_rotation,
    config.fund_member_limit,
    config.funding_goal,
    config.accepted_tokens,
    config.axelar_gateway,
    config.axelar_ibc_channel
  );
  await turnOverApTeamMultisig();
  // await createIndexFunds();
}

async function setup(
  tax_rate: string,
  treasury_address: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  fund_rotation: number | undefined,
  fund_member_limit: number | undefined,
  funding_goal: string | undefined,
  accepted_tokens: any | undefined,
  axelar_gateway: string,
  axelar_ibc_channel: string
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each and instantiate the contracts
  registrar = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    "registrar.wasm",
    {
      tax_rate,
      treasury: treasury_address,
      split_to_liquid: undefined,
      accepted_tokens: accepted_tokens,
      axelar_gateway,
      axelar_ibc_channel,
    }
  );
  cw4Group = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/cw4_group.wasm`
  );
  console.log("\nSetting up CW4 Group for AP Team");
  const cw4GrpApTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw4Group,
    {
      admin: apTeamAddr,
      members: [
        { addr: apTeamAddr, weight: 1 },
        { addr: apTeam2Addr, weight: 1 },
      ],
    }
  );
  cw4GrpApTeam = cw4GrpApTeamResult.contractAddress as string;
  cw3ApTeam = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    "cw3_apteam.wasm",
    {
      registrar_contract: registrar,
      group_addr: cw4GrpApTeam,
      threshold: {
        absolute_percentage: { percentage: threshold_absolute_percentage },
      },
      max_voting_period: { height: max_voting_period_height },
      // registrar_contract: registrar,
    }
  );
  // Setup AP Team C3 to be the admin to it's C4 Group
  process.stdout.write(
    "AddHook & UpdateAdmin on AP Team CW4 Group to point to AP Team C3"
  );
  await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
    add_hook: { addr: cw3ApTeam },
  });
  await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
    update_admin: { admin: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));
  settingsController = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    cw3ApTeam,
    "accounts_settings_controller.wasm",
    {
      owner_sc: cw3ApTeam,
      registrar_contract: registrar,
    }
  );
  accounts = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    cw3ApTeam,
    "accounts.wasm",
    {
      owner_sc: cw3ApTeam,
      registrar_contract: registrar,
    }
  );
  indexFund = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    cw3ApTeam,
    "index_fund.wasm",
    {
      registrar_contract: registrar,
      fund_rotation: fund_rotation,
      fund_member_limit: fund_member_limit,
      funding_goal: funding_goal,
    }
  );
  cw3MultiSigEndowment = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/cw3_endowment.wasm`
  );
  console.log("\nSetting up CW4 Group for AP Team");
  const cw4GrpReviewTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw4Group,
    {
      admin: apTeamAddr,
      members: [
        { addr: apTeamAddr, weight: 1 },
        { addr: apTeam2Addr, weight: 1 },
      ],
    }
  );
  cw4GrpReviewTeam = cw4GrpReviewTeamResult.contractAddress as string;
  cw3ReviewTeam = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    "cw3_applications.wasm",
    {
      registrar_contract: registrar,
      group_addr: cw4GrpReviewTeam,
      threshold: {
        absolute_percentage: { percentage: threshold_absolute_percentage },
      },
      max_voting_period: { height: max_voting_period_height },
      seed_split_to_liquid: "0.5",
      seed_asset: {
        info: { native: "ujunox" },
        amount: "100000",
      },
      new_endow_gas_money: { denom: "ujunox", amount: "100000" },
    }
  );
  // Setup AP Team C3 to be the admin to it's C4 Group
  process.stdout.write(
    "AddHook & UpdateAdmin on AP Review Team CW4 Group to point to AP Team C3"
  );
  await sendTransaction(juno, apTeamAddr, cw4GrpReviewTeam, {
    add_hook: { addr: cw3ReviewTeam },
  });
  await sendTransaction(juno, apTeamAddr, cw4GrpReviewTeam, {
    update_admin: { admin: cw3ReviewTeam },
  });
  console.log(chalk.green(" Done!"));

  swapRouter = await storeAndInstantiateContract(
    juno,
    apTeamAddr,
    cw3ApTeam,
    "swap_router.wasm",
    {
      registrar_contract: registrar,
      accounts_contract: accounts,
      pairs: [
        {
          assets: [
            {
              native: localjuno.networkInfo.nativeToken,
            },
            {
              cw20: localjuno.loopswap.malo_token_contract,
            },
          ],
          contract_address: localjuno.loopswap.malo_juno_pair_contract,
        },
        {
          assets: [
            {
              native: localjuno.networkInfo.nativeToken,
            },
            {
              cw20: localjuno.loopswap.kalo_token_contract,
            },
          ],
          contract_address: localjuno.loopswap.kalo_juno_pair_contract,
        },
        {
          assets: [
            {
              cw20: localjuno.loopswap.malo_token_contract,
            },
            {
              cw20: localjuno.loopswap.kalo_token_contract,
            },
          ],
          contract_address: localjuno.loopswap.malo_kalo_pair_contract,
        },
        {
          assets: [
            {
              native: localjuno.networkInfo.nativeToken,
            },
            {
              cw20: localjuno.loopswap.loop_token_contract,
            },
          ],
          contract_address: localjuno.loopswap.loop_juno_pair_contract,
        },
      ],
    }
  );
  // await storeAndInstantiateContract(juno, apTeamAddr, apTeamAddr, 'subdao.wasm', {});
  // await storeAndInstantiateContract(juno, apTeamAddr, apTeamAddr, 'subdao_bonding_token.wasm', {});
  // donationMatching = await storeAndInstantiateContract(
  //   juno,
  //   apTeamAddr,
  //   cw3ApTeam,
  //   "donation_match.wasm",
  //   {
  //     id: 1, // FAKE! Need to fix.
  //     registrar_contract: registrar,
  //     reserve_token: apTeamAddr, // FAKE! Need to fix.
  //     lp_pair: apTeamAddr, // FAKE! Need to fix.
  //   }
  // );

  process.stdout.write(
    "Update Registrar's config with various wasm codes & contracts"
  );
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_config: {
      accounts_contract: accounts,
      accounts_settings_controller: settingsController,
      applications_review: cw3ReviewTeam,
      index_fund_contract: indexFund,
      cw3_code: cw3MultiSigEndowment,
      cw4_code: cw4Group,
      halo_token: apTeamAddr, // Fake halo token addr: Need to be handled
      halo_token_lp_contract: apTeamAddr, // Fake halo token LP addr: Need to be handled
      subdao_gov_code: undefined,
      subdao_cw20_token_code: undefined,
      subdao_bonding_token_code: undefined,
      donation_match_code: undefined,
      donation_match_charites_contract: undefined,
    },
  });
  console.log(chalk.green(" Done!"));
  
  process.stdout.write(
    "Top up the AP Applications CW3 with wallet dusting & seed contribution tokens"
  );
  await juno.sendTokens(apTeamAddr, cw3ReviewTeam, [{ denom: "ujunox", amount: "100000" }], "auto", "initial dust & seed funds");
  console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(): Promise<void> {
  process.stdout.write(
    "Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract\n"
  );
  process.stdout.write(chalk.yellow("\n> Turning over Registrar"));
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_owner: { new_owner: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));
}
