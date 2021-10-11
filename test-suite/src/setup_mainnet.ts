/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  migrateContract,
  sendTransaction,
  storeCode,
  instantiateContract,
} from "./helpers";
import * as mainNet from "./charities";

chai.use(chaiAsPromised);

type Member = {
  addr: string,
  weight: number
};

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

let terra: LCDClient;
let apTeam: Wallet;

let accountsCodeId: number;
let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault: string;
let anchorMoneyMarket: string;
let endowmentContracts: string[];

export function initializeLCDClient(
  lcdClient: LCDClient,
  ap_team: Wallet,
  anchorMoneyMarketAddr: string): void {

  terra = lcdClient;
  apTeam = ap_team;
  anchorMoneyMarket = anchorMoneyMarketAddr;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);

  registrar = "terra10dw3u9can2gwtg57wdazedtqa5ua93fxhy39jr";
  indexFund = "terra1aeyu80927kaa0a57zawnv7hza73pd7jm0u6c79";
  anchorVault = "terra16qdyngrfh3ssh80ylsspu4pax9gkygu3hddre2";
  cw4GrpApTeam = "terra18merrvpm9l0hl86yfhz3lyzzysv3fp4xspy7h2";
  cw3ApTeam = "terra1nx4p4nkrxq5q4yuryl5mpumhxulveyrywg7yx3";
  cw4GrpOwners = "terra1erd5uhnqurlvuvgeve2apd99c6l2h653lv28cj";
  cw3GuardianAngels = "terra19e8sr3hgakqngd996phy457lf4k69v895e5yqf";
  endowmentContracts = [
    "terra1v2tqeeag8gvn4mgglhre9z28g8vuz53llewxxs",
    "terra1j4layl9zf6uyeea443d4jqgj92scm7gfxnxtqg",
    "terra154e4fj5nhgtd4y7e39fkgkfjs4z68dwdnjfv83",
    "terra1nvk7w5mnzer9qktkvj0pm4k3qr0lvwq6q3ymcy",
    "terra12gycny6knqvmceahqtulfknfwuldff2fqsg6ec",
    "terra1ceqavxry7w6v65wup8t3jlwfzqqph240gcv9dl",
    "terra1cl7m9slyll8v2jjg3ehjtgn0hvpnvd33daf49q",
    "terra1ydx8250f785u28ms9ukzsknhu0jnfvhf3jg3j6",
    "terra120ek7dtx53xyny3p9klsvymgsnzsdx7l37ca3j",
    "terra15w7tunh0tgv3ve8fyyu4hhykdu62c44uc34vza",
    "terra1hzv0k35dk95qx0gculh2t0rrkdsaqd3fhlrh96",
    "terra1rj30h0dvvx9aqa5hsjux5ts4gg09zuwx7jnszj",
    "terra1drldgctf4pkgx7efdzrfuewxejmygfwnysgkrw",
    "terra1y2r7xwwfmkxqcvxh5r5jhpgh2qqp7gus49f42n",
    "terra15nzpmd5pr7u6xr0rrp80p4vs0ga62aywm2yh00"
  ]

  console.log(`Use ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Use ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Use ${chalk.cyan(anchorVault)} as Anchor Vault`);
  console.log(`Use ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Use ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Use ${chalk.cyan(cw4GrpOwners)} as CW4 Endowment Owners Group`);
  console.log(`Use ${chalk.cyan(cw3GuardianAngels)} as CW3 Guardian Angels MultiSig`);
}

//----------------------------------------------------------------------------------------
// Setup Contracts for MainNet
//----------------------------------------------------------------------------------------

export async function setupContractsForMainNet(
  treasury_address: string,
  members: Member[],
  tca_members: string[],
  tax_rate: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  max_voting_period_guardians_height: number,
  fund_rotation: number | undefined,
  turnover_to_multisig: boolean,
  harvest_to_liquid: string,
  tax_per_block: string,
  funding_goal: string | undefined,
): Promise<void> {
  await setupContracts(
    treasury_address,
    members,
    tca_members,
    tax_rate,
    threshold_absolute_percentage,
    max_voting_period_height,
    max_voting_period_guardians_height,
    fund_rotation,
    harvest_to_liquid,
    tax_per_block,
    funding_goal
  );
  await mainNet.initializeCharities(terra, apTeam, registrar, indexFund);
  await mainNet.setupEndowments();
  await mainNet.approveEndowments();
  await mainNet.createIndexFunds();
}

async function setupContracts(
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
    path.resolve(__dirname, "../../artifacts/registrar.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);
  
  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/anchor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);
  
  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/index_fund.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);
  
  process.stdout.write("Uploading Accounts Wasm");
  accountsCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/accounts.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

  process.stdout.write("Uploading CW4 Group Wasm");
  const cw4Group = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/cw4_group.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

  process.stdout.write("Uploading Guardian Angels MultiSig Wasm");
  const guardianAngelMultiSig = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/guardian_angels_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${guardianAngelMultiSig}`);

  process.stdout.write("Uploading AP Team MultiSig Wasm");
  const apTeamMultiSig = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/ap_team_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${apTeamMultiSig}`);

  // Step 2. Instantiate the key contracts
  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(terra, apTeam, apTeam, registrarCodeId, {
    accounts_code_id: accountsCodeId,
    treasury: treasury_address,
    tax_rate: tax_rate,
    default_vault: undefined,
  });
  registrar = registrarResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // CW4 AP Team Group
  process.stdout.write("Instantiating CW4 AP Team Group contract");
  const cw4GrpApTeamResult = await instantiateContract(terra, apTeam, apTeam, cw4Group, {
    admin: apTeam.key.accAddress,
    members: members,
  });
  cw4GrpApTeam = cw4GrpApTeamResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpApTeam}`);

  // CW3 AP Team MultiSig
  process.stdout.write("Instantiating CW3 AP Team MultiSig contract");
  const cw3ApTeamResult = await instantiateContract(terra, apTeam, apTeam, apTeamMultiSig, {
    group_addr: cw4GrpApTeam,
    threshold: { absolute_percentage: { percentage: threshold_absolute_percentage }},
    max_voting_period: { height: max_voting_period_height },
    registrar_contract: registrar,
  });
  cw3ApTeam = cw3ApTeamResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3ApTeam}`);

  // Setup AP Team C3 to be the admin to it's C4 Group 
  process.stdout.write("AddHook & UpdateAdmin on AP Team CW4 Group to point to AP Team C3");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, cw4GrpApTeam, {
      add_hook: { addr: cw3ApTeam }
    }),
    new MsgExecuteContract(apTeam.key.accAddress, cw4GrpApTeam, {
      update_admin: { admin: cw3ApTeam }
    }),
  ]);
  console.log(chalk.green(" Done!")); 

  // CW4 Endowment Owners Group
  // Registrar SC is the Admin & no members in the group to start
  process.stdout.write("Instantiating CW4 Endowment Owners Group contract");
  const cw4GrpOwnersResult = await instantiateContract(terra, apTeam, apTeam, cw4Group, {
    admin: registrar,
    members: [],
  });
  cw4GrpOwners = cw4GrpOwnersResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpOwners}`);

  // CW3 Guardian Angels MultiSig
  process.stdout.write("Instantiating CW3 Guardian Angels MultiSig contract");
  const cw3Result = await instantiateContract(terra, apTeam, apTeam, guardianAngelMultiSig, {
    ap_team_group: cw4GrpApTeam,
    endowment_owners_group: cw4GrpOwners,
    registrar_contract: registrar,
    threshold: { absolute_percentage: { percentage: threshold_absolute_percentage }},
    max_voting_period: { height: max_voting_period_height },
    max_voting_period_guardians: { height: max_voting_period_guardians_height },
  });
  cw3GuardianAngels = cw3Result.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw3GuardianAngels}`);


  // Update the Registrar with newly created Endowment Owners Group & Guardians Multisig address
  process.stdout.write("Update Registrar with the Address of the CW4 Endowment Owners Group contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        endowment_owners_group_addr: cw4GrpOwners,
        guardians_multisig_addr: cw3GuardianAngels,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(terra, apTeam, apTeam, fundCodeId, {
    registrar_contract: registrar,
    fund_rotation: fund_rotation,
    funding_goal: funding_goal,
  });
  indexFund = fundResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
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
    harvest_to_liquid: harvest_to_liquid
  });
  anchorVault = vaultResult1.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault in Registrar");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: { 
        vault_addr: anchorVault,
        approved: true,
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("Set default vault in Registrar (for newly created Endowments) as Anchor Vault #1");
  process.stdout.write("Update Registrar with the Address of the Index Fund contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        default_vault: anchorVault,
        index_fund_contract: indexFund,
      }
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

//----------------------------------------------------------------------------------------
// Migration contracts
//----------------------------------------------------------------------------------------
export async function migrateContracts(): Promise<void> {
  // run the migrations desired
  // await migrateRegistrar();
  // await migrateCw4Group();
  // await migrateApTeamMultisig();
  // await migrateGuardianAngelsMultisig();
  // await migrateIndexFund();
  await migrateAccounts();
  // await migrateVaults();
}

// -------------------------------------------------
//  Base functions to migrate contracts with 
//--------------------------------------------------
async function migrateRegistrar() {
  process.stdout.write("Uploading Registrar Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/registrar.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  process.stdout.write("Migrate Registrar contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, registrar, codeId, {});
  console.log(chalk.green(" Done!"));
}

async function migrateCw4Group() {
  process.stdout.write("Uploading CW4 Group Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/cw4_group.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate CW4 AP Team Group contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw4GrpApTeam, codeId, {});
  console.log(chalk.green(" Done!"));

  process.stdout.write("Migrate CW4 Endowment Owners Group contract");
  const result2 = await migrateContract(terra, apTeam, apTeam, cw4GrpOwners, codeId, {});
  console.log(chalk.green(" Done!"));
}

async function migrateApTeamMultisig() {
  process.stdout.write("Uploading AP Team MultiSig Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/ap_team_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate AP Team MultiSig contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw3ApTeam, codeId, {});
  console.log(chalk.green(" Done!"));
}

async function migrateGuardianAngelsMultisig() {
  process.stdout.write("Uploading Guardian Angels MultiSig Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/guardian_angels_multisig.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Guardian Angels MultiSig contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, cw3GuardianAngels, codeId, {});
  console.log(chalk.green(" Done!"));
}

async function migrateIndexFund() {
  process.stdout.write("Uploading Index Fund Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/index_fund.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  process.stdout.write("Migrate Index Fund contract");
  const result1 = await migrateContract(terra, apTeam, apTeam, indexFund, codeId, {});
  console.log(chalk.green(" Done!"));
}

async function migrateVaults() {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/anchor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  await migrateContract(terra, apTeam, apTeam, anchorVault, codeId, {});
  console.log(chalk.green(`anchorVault - Done!`));
}

async function migrateAccounts() {
  process.stdout.write("Uploading Accounts Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/accounts.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);
  
  // Update registrar accounts code ID and migrate all accounts contracts
  process.stdout.write("Update Registrar's Account Code ID stored in configs");
  const result0 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: { accounts_code_id: codeId }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("Migrate Accounts contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  endowmentContracts.forEach(endowment => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(() => new Promise(async (resolve, reject) => {
      try {
        await migrateContract(terra, apTeam, apTeam, endowment, codeId, {});
        console.log(chalk.green(`#${id ++} - Done!`));
        resolve();
      } catch(e) {
        reject(e);
      }
    }));
  });

  await prom;
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryRegistrarConfig(): Promise<void> {
  process.stdout.write("Test - Query Registrar config and get proper result");
  const result: any = await terra.wasm.contractQuery(registrar, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentList(): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    endowment_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultList(): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedVaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultRateList(): Promise<void> {
  process.stdout.write("Test - Query Registrar Approved Vault Exchange Rate List");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_rate_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVaultList(): Promise<void> {
  process.stdout.write("Test - Query Registrar VaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVault(): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault: {
      vault_addr: anchorVault,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundConfig(): Promise<void> {
  process.stdout.write("Test - Query IndexFund Config");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    config: {},
  });
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundState(): Promise<void> {
  process.stdout.write("Test - Query IndexFund State");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    state: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundTcaList(): Promise<void> {
  process.stdout.write("Test - Query IndexFund TcaList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    tca_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundsList(): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundsList");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    funds_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundFundDetails(): Promise<void> {
  process.stdout.write("Test - Query IndexFund FundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    fund_details: { fund_id: 1 },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDetails(): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDetails");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_details: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryIndexFundActiveFundDonations(): Promise<void> {
  process.stdout.write("Test - Query IndexFund ActiveFundDonations");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    active_fund_donations: {},
  });

  console.log(result);
  console.log(chalk.green("Passed!"));
}

export async function testQueryIndexFundDeposit(): Promise<void> {
  process.stdout.write("Test - Query IndexFund Deposit msg builder");
  const result: any = await terra.wasm.contractQuery(indexFund, {
    deposit: {
      amount: "100000000",
      fund_id: undefined
    },
  });

  console.log(result);
  console.log(chalk.green("Passed!"));
}

