/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import * as mainNet from "./charities";
import { 
  sendTransaction, 
  storeCode, 
  instantiateContract, 
  getWalletAddress, 
  Member 
} from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let juno: SigningCosmWasmClient;
let apTeam: string;
let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault: string;

//----------------------------------------------------------------------------------------
// Setup Contracts for MainNet
//----------------------------------------------------------------------------------------

export async function setupCore(
  _juno: SigningCosmWasmClient,
  _apTeam: string,
  treasury_address: string,
  members: Member[],
  tca_members: string[],
  config: {
    tax_rate: string;
    threshold_absolute_percentage: string;
    max_voting_period_height: number;
    max_voting_period_guardians_height: number;
    fund_rotation: number | undefined;
    turnover_to_multisig: boolean;
    is_localjuno: boolean;
    harvest_to_liquid: string;
    tax_per_block: string;
    funding_goal: string | undefined;
    fund_member_limit: undefined, // fund_member_limit
    accepted_tokens: undefined,  // accepted_tokens for "index_fund"
    charity_cw3_multisig_threshold_abs_perc: string,
    charity_cw3_multisig_max_voting_period: number,
  }
): Promise<void> {
  // Initialize variables
  juno = _juno;
  apTeam = _apTeam;

  await setup(
    treasury_address,
    members,
    tca_members,
    config.tax_rate,
    config.threshold_absolute_percentage,
    config.max_voting_period_height,
    config.max_voting_period_guardians_height,
    config.fund_rotation,
    config.harvest_to_liquid,
    config.tax_per_block,
    config.funding_goal,
    config.fund_member_limit,
    config.accepted_tokens,
  );
  await mainNet.initializeCharities(juno, apTeam, registrar, indexFund);
  await mainNet.setupEndowments(
    config.charity_cw3_multisig_threshold_abs_perc,
    config.charity_cw3_multisig_max_voting_period,
  );
  await mainNet.approveEndowments();
  await mainNet.createIndexFunds();
}

async function setup(
  treasury_address: string,
  members: Member[],
  tca_members: string[],
  tax_rate: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  max_voting_period_guardians_height: number,
  fund_rotation: number | undefined,
  harvest_to_liquid: string,
  tax_per_block: string,
  funding_goal: string | undefined,
  fund_member_limit: number | undefined,
  accepted_tokens: any | undefined,
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each....
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/registrar.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(juno, apTeam, `${wasm_path.core}/anchor.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(juno, apTeam, `${wasm_path.core}/index_fund.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);

  process.stdout.write("Uploading Accounts Wasm");
  const accountsCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/accounts.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

  process.stdout.write("Uploading CW4 Group Wasm");
  const cw4Group = await storeCode(juno, apTeam, `${wasm_path.core}/cw4_group.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

  process.stdout.write("Uploading CW3 MultiSig Wasm");
  const cw3MultiSig = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/cw3_multisig.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSig}`);

  process.stdout.write("Uploading AP Team MultiSig Wasm");
  const apTeamMultiSig = await storeCode(
    juno,
    apTeam,
    `${wasm_path.core}/cw3_multisig.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${apTeamMultiSig}`);

  // Step 2. Instantiate the key contracts
  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(
    juno,
    apTeam,
    apTeam,
    registrarCodeId,
    {
      accounts_code_id: accountsCodeId,
      treasury: treasury_address,
      tax_rate: tax_rate,
      default_vault: undefined,
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujunox'],
        cw20: [],
      }
    }
  );
  registrar = registrarResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // CW4 AP Team Group
  process.stdout.write("Instantiating CW4 AP Team Group contract");
  const cw4GrpApTeamResult = await instantiateContract(juno, apTeam, apTeam, cw4Group, {
    admin: apTeam,
    members: members,
  });
  cw4GrpApTeam = cw4GrpApTeamResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpApTeam}`);

  // CW3 AP Team MultiSig
  process.stdout.write("Instantiating CW3 AP Team MultiSig contract");
  const cw3ApTeamResult = await instantiateContract(
    juno,
    apTeam,
    apTeam,
    apTeamMultiSig,
    {
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
  await sendTransaction(juno, apTeam, cw4GrpApTeam, {
    add_hook: { addr: cw3ApTeam },
  });
  await sendTransaction(juno, apTeam, cw4GrpApTeam, {
    update_admin: { admin: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(juno, apTeam, apTeam, fundCodeId, {
    registrar_contract: registrar,
    fund_rotation: fund_rotation,
    fund_member_limit: fund_member_limit,
    funding_goal: funding_goal,
    accepted_tokens: accepted_tokens,
  });
  indexFund = fundResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);

  // Anchor Vault
  process.stdout.write("Instantiating Anchor Vault contract");
  const vaultResult1 = await instantiateContract(juno, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP Deposit Token - Anchor",
    symbol: "apANC",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  anchorVault = vaultResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault in Registrar");
  process.stdout.write(
    "Set default vault in Registrar (for newly created Endowments) as Anchor Vault"
  );
  process.stdout.write("Update Registrar with the Address of the Index Fund contract,  CW3_code_Id, CW4_code_Id");
  await sendTransaction(juno, apTeam, registrar, {
    update_config: {
      default_vault: anchorVault,
      index_fund_contract: indexFund,
      cw3_code: cw3MultiSig,
      cw4_code: cw4Group,
    },
  });
  await sendTransaction(juno, apTeam, registrar, {
    vault_update_status: {
      vault_addr: anchorVault,
      approved: true,
    },
  });
  console.log(chalk.green(" Done!"));

  // Add confirmed TCA Members to the Index Fund SCs approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  await sendTransaction(juno, apTeam, indexFund, {
    update_tca_list: { new_list: tca_members },
  });
  console.log(chalk.green(" Done!"));
}
