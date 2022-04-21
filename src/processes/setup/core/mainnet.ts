/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, storeCode, instantiateContract } from "../../../utils/helpers";
import * as mainNet from "./charities";
import { wasm_path } from "../../../config/wasmPaths";

chai.use(chaiAsPromised);

export type Member = {
  addr: string;
  weight: number;
};

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let terra: LCDClient;
let apTeam: Wallet;

let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault: string;
let anchorMoneyMarket: string;
//----------------------------------------------------------------------------------------
// Setup Contracts for MainNet
//----------------------------------------------------------------------------------------

export async function setupCore(
  _terra: LCDClient,
  _apTeam: Wallet,
  _anchorMoneyMarket: string,
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
    is_localterra: boolean;
    harvest_to_liquid: string;
    tax_per_block: string;
    funding_goal: string | undefined;
  }
): Promise<void> {
  // Initialize variables
  terra = _terra;
  apTeam = _apTeam;
  anchorMoneyMarket = _anchorMoneyMarket;

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
    config.funding_goal
  );
  await mainNet.initializeCharities(terra, apTeam, registrar, indexFund);
  await mainNet.setupEndowments();
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
  funding_goal: string | undefined
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each....
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.core}/registrar.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(terra, apTeam, `${wasm_path.core}/anchor.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(terra, apTeam, `${wasm_path.core}/index_fund.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);

  process.stdout.write("Uploading Accounts Wasm");
  const accountsCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.core}/accounts.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

  process.stdout.write("Uploading CW4 Group Wasm");
  const cw4Group = await storeCode(terra, apTeam, `${wasm_path.core}/cw4_group.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

  process.stdout.write("Uploading Guardian Angels MultiSig Wasm");
  const guardianAngelMultiSig = await storeCode(
    terra,
    apTeam,
    `${wasm_path.core}/cw3_multisig.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${guardianAngelMultiSig}`);

  // Step 2. Instantiate the key contracts
  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(
    terra,
    apTeam,
    apTeam,
    registrarCodeId,
    {
      accounts_code_id: accountsCodeId,
      treasury: treasury_address,
      tax_rate: tax_rate,
      default_vault: undefined,
    }
  );
  registrar = registrarResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(terra, apTeam, apTeam, fundCodeId, {
    registrar_contract: registrar,
    fund_rotation: fund_rotation,
    funding_goal: funding_goal,
  });
  indexFund = fundResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);

  // Anchor Vault
  process.stdout.write("Instantiating Anchor Vault contract");
  const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP Deposit Token - Anchor",
    symbol: "apANC",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  anchorVault = vaultResult1.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault in Registrar");
  process.stdout.write(
    "Set default vault in Registrar (for newly created Endowments) as Anchor Vault"
  );
  process.stdout.write("Update Registrar with the Address of the Index Fund contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        default_vault: anchorVault,
        index_fund_contract: indexFund,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: {
        vault_addr: anchorVault,
        approved: true,
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Add confirmed TCA Members to the Index Fund SCs approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_tca_list: { new_list: tca_members },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}
