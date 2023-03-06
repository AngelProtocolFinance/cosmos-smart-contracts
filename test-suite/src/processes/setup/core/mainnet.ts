/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import {
  sendTransaction,
  storeCode,
  instantiateContract,
  getWalletAddress,
  Member,
} from "../../../utils/juno/helpers";
import { wasm_path } from "../../../config/wasmPaths";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let juno: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeamAddr: string;

let registrar: string;
let accounts: string;
let indexFund: string;
let settingsController: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;

let donationMatchCharities: string;
let vault1: string;

//----------------------------------------------------------------------------------------
// Setup Contracts for MainNet
//----------------------------------------------------------------------------------------

export async function setupCore(
  _juno: SigningCosmWasmClient,
  _apTeam: DirectSecp256k1HdWallet,
  treasury_address: string,
  config: {
    tax_rate: string;
    threshold_absolute_percentage: string;
    max_voting_period_height: number;
    fund_rotation: number | undefined;
    fund_member_limit: number | undefined;
    funding_goal: string | undefined;
    accepted_tokens: any | undefined;
  }
): Promise<void> {
  // Initialize variables
  juno = _juno;
  apTeam = _apTeam;
  apTeamAddr = await getWalletAddress(apTeam);

  await setup(
    treasury_address,
    config.tax_rate,
    config.threshold_absolute_percentage,
    config.max_voting_period_height,
    config.fund_rotation,
    config.fund_member_limit,
    config.funding_goal,
    config.accepted_tokens
  );
  await turnOverApTeamMultisig();
}

async function setup(
  treasury_address: string,
  tax_rate: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  fund_rotation: number | undefined,
  fund_member_limit: number | undefined,
  funding_goal: string | undefined,
  accepted_tokens: any | undefined
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each....
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/registrar.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/index_fund.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);

  process.stdout.write("Uploading Accounts Wasm");
  const accountsCodeId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/accounts.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

  process.stdout.write("Uploading Settings-Controller Wasm");
  const settingsControllerCodId = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/settings_controller.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${settingsControllerCodId}`
  );

  process.stdout.write("Uploading CW4 Group Wasm");
  const cw4Group = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw4_group.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

  process.stdout.write("Uploading AP Team CW3 MultiSig Wasm");
  const cw3MultiSigApTeam = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/cw3_apteam.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigApTeam}`);

  process.stdout.write("Uploading Review Team CW3 MultiSig Wasm");
  const cw3MultiSigApplications = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/cw3_applications.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${cw3MultiSigApplications}`
  );

  process.stdout.write("Uploading Endowment CW3 MultiSig Wasm");
  const cw3MultiSigEndowment = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/cw3_endowment.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigEndowment}`);

  process.stdout.write("Uploading Endowment SubDAO Wasm");
  const subdao = await storeCode(juno, apTeamAddr, `${wasm_path.core}/subdao.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdao}`);

  process.stdout.write("Uploading Endowment SubDAO Token Wasm");
  const subdaoToken = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/subdao_token.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoToken}`);

  process.stdout.write("Uploading Endowment SubDAO Donation Matching Wasm");
  const subdaoDonationMatch = await storeCode(
    juno,
    apTeamAddr,
    `${wasm_path.core}/donation_match.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoDonationMatch}`);

  // Step 2. Instantiate the key contracts
  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    registrarCodeId,
    {
      tax_rate,
      treasury: treasury_address,
      split_to_liquid: undefined,
      rebalance: undefined,
      accepted_tokens: accepted_tokens,
    }
  );
  registrar = registrarResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, fundCodeId, {
    registrar_contract: registrar,
    fund_rotation: fund_rotation,
    fund_member_limit: fund_member_limit,
    funding_goal: funding_goal,
    accepted_tokens: accepted_tokens,
  });
  indexFund = fundResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);

  // CW4 AP Team Group
  process.stdout.write("Instantiating CW4 AP Team Group contract");
  const cw4GrpApTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw4Group,
    {
      admin: apTeamAddr,
      members: [{ addr: apTeamAddr, weight: 1 }],
    }
  );
  cw4GrpApTeam = cw4GrpApTeamResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpApTeam}`);

  // CW3 AP Team MultiSig
  process.stdout.write("Instantiating CW3 AP Team MultiSig contract");
  const cw3ApTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw3MultiSigApTeam,
    {
      registrar_contract: registrar,
      group_addr: cw4GrpApTeam,
      threshold: { absolute_percentage: { percentage: threshold_absolute_percentage } },
      max_voting_period: { height: max_voting_period_height },
    }
  );
  cw3ApTeam = cw3ApTeamResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3ApTeam}`);

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

  process.stdout.write("Instantiating Settings-Controller contract");
  const settingsControllerResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    settingsControllerCodId,
    {
      owner_sc: cw3ApTeam,
      registrar_contract: registrar,
    }
  );
  settingsController = settingsControllerResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${settingsController}`
  );

  process.stdout.write("Instantiating Accounts contract");
  const accountsResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    accountsCodeId,
    {
      owner_sc: cw3ApTeam,
      registrar_contract: registrar,
    }
  );
  accounts = accountsResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${accounts}`);

  // CW4 Review Team Group
  process.stdout.write("Instantiating CW4 Review Team Group contract");
  const cw4GrpReviewTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw4Group,
    {
      admin: apTeamAddr,
      members: [{ addr: apTeamAddr, weight: 1 }],
    }
  );
  cw4GrpReviewTeam = cw4GrpReviewTeamResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${cw4GrpReviewTeam}`
  );

  // CW3 Review Team MultiSig
  process.stdout.write("Instantiating CW3 Review Team MultiSig contract");
  const cw3ReviewTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw3MultiSigApplications,
    {
      registrar_contract: registrar,
      group_addr: cw4GrpReviewTeam,
      threshold: { absolute_percentage: { percentage: threshold_absolute_percentage } },
      max_voting_period: { height: max_voting_period_height },
    }
  );
  cw3ReviewTeam = cw3ReviewTeamResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3ReviewTeam}`);

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

  process.stdout.write("Update Registrar's config with various wasm codes & contracts");
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_config: {
      accounts_contract: accounts,
      applications_review: cw3ReviewTeam,
      index_fund_contract: indexFund,
      cw3_code: cw3MultiSigEndowment,
      cw4_code: cw4Group,
      halo_token: apTeam, // Fake halo token addr: Need to be handled
      halo_token_lp_contract: apTeam, // Fake halo token LP addr: Need to be handled
      subdao_gov_code: subdao,
      subdao_token_code: subdaoToken,
      donation_match_code: subdaoDonationMatch,
      donation_match_charites_contract: donationMatchCharities,
    },
  });
  console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(): Promise<void> {
  process.stdout.write(
    "Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract\n"
  );
  process.stdout.write(chalk.yellow("- Turning over Registrar"));
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_owner: { new_owner: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write(chalk.yellow("- Turning over Index Fund"));
  await sendTransaction(juno, apTeamAddr, indexFund, {
    update_owner: { new_owner: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));
}
