/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import {
  sendTransaction,
  storeCode,
  instantiateContract,
  migrateContract,
  toEncodedBinary,
} from "./helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

let terra: LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;
let charity1: Wallet;
let charity2: Wallet;
let charity3: Wallet;
let pleb: Wallet;
let tca: Wallet;

let accountsCodeId: number;
let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault1: string;
let anchorVault2: string;
let anchorMoneyMarket: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;
let endowmentContract4: string;

// Anchor aUST Token
const yieldToken = "terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl";

//----------------------------------------------------------------------------------------
// Initialize variables
//----------------------------------------------------------------------------------------
export function initializeLocalTerra(localTerra: LocalTerra): void {
  terra = localTerra;
  apTeam = localTerra.wallets.test1;
  charity1 = localTerra.wallets.test2;
  charity2 = localTerra.wallets.test3;
  charity3 = localTerra.wallets.test4;
  pleb = localTerra.wallets.test5;
  tca = localTerra.wallets.test6;
  apTeam2 = localTerra.wallets.test7;
  apTeam3 = localTerra.wallets.test8;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team #1`);
  console.log(`Use ${chalk.cyan(apTeam2.key.accAddress)} as Angel Team #2`);
  console.log(`Use ${chalk.cyan(apTeam3.key.accAddress)} as Angel Team #3`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);
}

export function initializeLCDClient(
  lcdClient: LCDClient,
  wallets: {
    apTeam: Wallet,
    apTeam2: Wallet,
    apTeam3: Wallet,
    charity1: Wallet,
    charity2: Wallet,
    charity3: Wallet,
    pleb: Wallet,
    tca: Wallet
  },
  anchorMoneyMarketAddr: string): void {
  terra = lcdClient;
  apTeam = wallets.apTeam;
  apTeam2 = wallets.apTeam2;
  apTeam3 = wallets.apTeam3;
  charity1 = wallets.charity1;
  charity2 = wallets.charity2;
  charity3 = wallets.charity3;
  pleb = wallets.pleb;
  tca = wallets.tca;
  anchorMoneyMarket = anchorMoneyMarketAddr;

  console.log(`Use ${chalk.cyan(apTeam.key.accAddress)} as Angel Team`);
  console.log(`Use ${chalk.cyan(apTeam2.key.accAddress)} as Angel Team #2`);
  console.log(`Use ${chalk.cyan(apTeam3.key.accAddress)} as Angel Team #3`);
  console.log(`Use ${chalk.cyan(charity1.key.accAddress)} as Charity #1`);
  console.log(`Use ${chalk.cyan(charity2.key.accAddress)} as Charity #2`);
  console.log(`Use ${chalk.cyan(charity3.key.accAddress)} as Charity #3`);
  console.log(`Use ${chalk.cyan(pleb.key.accAddress)} as Pleb`);
  console.log(`Use ${chalk.cyan(tca.key.accAddress)} as TCA member`);

  registrar = "terra1swwyv5xz5kvjgy3pgykdp9t8jj94kt8y3yepn6";
  indexFund = "terra1agxu6rvnmxwvxkgfmdfg4rpdwtg6zftmnwf9uk";
  anchorVault1 = "terra1zgg4kx0f9surr7djwfedh7ecl6ky8dtwauamyl";
  anchorVault2 = "terra1qa8t7e0e7u6arv6vmjs8psc88gdektsj88g3vt";
  endowmentContract1 = "terra190pq8mrz44d4dmn45evhg30yf0k8lgkvj3pgvl";
  endowmentContract2 ="terra1mxf7nua6exs7r3cghyt7mrezw5teamchv0c5x2";
  endowmentContract3 = "terra1n39q6pxx43wqer89fk27j376prj3fjc5jw3veh";
  endowmentContract4 = "terra1ruapk3aythhfr56268wvu8a8lr8l7hztlwrdep";
  cw4GrpApTeam = "terra1yhsu3mkep8rj7dlseyqme56u89jg42saysa37r";
  cw3ApTeam = "terra1tyf0q9wsy2xu89466pdkuy5cjnkslej4x65hg4";
  cw4GrpOwners = "terra19u9cncja73k7rmyprny766q72q3luuew52mt8y";
  cw3GuardianAngels = "terra12ng433wdnq4psyx7xt9f9lkys26phxrjttep6a";

  console.log(`Use ${chalk.cyan(registrar)} as Registrar`);
  console.log(`Use ${chalk.cyan(indexFund)} as IndexFund`);
  console.log(`Use ${chalk.cyan(anchorVault1)} as Anchor Vault #1`);
  console.log(`Use ${chalk.cyan(anchorVault2)} as Anchor Vault #2`);
  console.log(`Use ${chalk.cyan(endowmentContract1)} as Endowment Contract #1`);
  console.log(`Use ${chalk.cyan(endowmentContract2)} as Endowment Contract #2`);
  console.log(`Use ${chalk.cyan(endowmentContract3)} as Endowment Contract #3`);
  console.log(`Use ${chalk.cyan(endowmentContract4)} as Endowment Contract #4`);
  console.log(`Use ${chalk.cyan(cw4GrpApTeam)} as CW4 AP Team Group`);
  console.log(`Use ${chalk.cyan(cw3ApTeam)} as CW3 AP Team MultiSig`);
  console.log(`Use ${chalk.cyan(cw4GrpOwners)} as CW4 Endowment Owners Group`);
  console.log(`Use ${chalk.cyan(cw3GuardianAngels)} as CW3 Guardian Angels MultiSig`);
}

// -----------------------------
// Migrate Vault contracts
// -----------------------------
export async function migrateContracts(): Promise<void> {
  // run the migrations desired
  await migrateRegistrar();
  await migrateCw4Group();
  await migrateApTeamMultisig();
  await migrateGuardianAngelsMultisig();
  await migrateIndexFund();
  await migrateAccounts();
  await migrateVaults();
}

// -------------------------------------------------
//  Base functions to migrate contracts with 
//--------------------------------------------------
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

async function migrateVaults() {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const codeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../../artifacts/anchor.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  await migrateContract(terra, apTeam, apTeam, anchorVault1, codeId, {});
  console.log(chalk.green(`anchorVault #1 - Done!`));
  await migrateContract(terra, apTeam, apTeam, anchorVault2, codeId, {});
  console.log(chalk.green(`anchorVault #2 - Done!`));
  // let counter = 1;
  // vaults.forEach(async function(vault) {
  //   setTimeout(async () => {
  //     await migrateContract(terra, apTeam, apTeam, vault, codeId, {});
  //     console.log(chalk.green(`#${counter} - Done!`));
  //     counter += 1;
  //   }, 7000);
  // });
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
  await migrateContract(terra, apTeam, apTeam, endowmentContract1, codeId, {});
  console.log(chalk.green(`#1 - Done!`));
  await migrateContract(terra, apTeam, apTeam, endowmentContract2, codeId, {});
  console.log(chalk.green(`#2 - Done!`));
  await migrateContract(terra, apTeam, apTeam, endowmentContract3, codeId, {});
  console.log(chalk.green(`#3 - Done!`));
  await migrateContract(terra, apTeam, apTeam, endowmentContract4, codeId, {});
  console.log(chalk.green(`#4 - Done!`));

  // let counter = 1;
  // accounts.forEach(async function(account) {
  //   setTimeout(async () => {
  //     await migrateContract(terra, apTeam, apTeam, account, codeId, {});
  //     console.log(chalk.green(`#${counter} - Done!`));
  //     counter += 1;
  //   }, 7000);
  // });

  // process.stdout.write("Migrate all Accounts contract via Registrar endpoint");
  // const charityResult1 = await sendTransaction(terra, apTeam, [
  //   new MsgExecuteContract(apTeam.key.accAddress, registrar, {
  //     migrate_accounts: {}
  //   }),
  // ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// Setup all contracts
//----------------------------------------------------------------------------------------

export async function setupContracts(): Promise<void> {
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
  // CW4 AP Team Group
  process.stdout.write("Instantiating CW4 AP Team Group contract");
  const cw4GrpApTeamResult = await instantiateContract(terra, apTeam, apTeam, cw4Group, {
    admin: apTeam.key.accAddress,
    members: [
      { addr: apTeam.key.accAddress, weight: 1 },
      { addr: apTeam2.key.accAddress, weight: 1 },
    ],
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
    threshold: { absolute_percentage: { percentage: "0.25" }},
    max_voting_period: { height: 100000 },
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

  // Registrar
  process.stdout.write("Instantiating Registrar contract");
  const registrarResult = await instantiateContract(terra, apTeam, apTeam, registrarCodeId, {
    accounts_code_id: accountsCodeId,
    treasury: apTeam.key.accAddress,
    tax_rate: "0.2",
    default_vault: undefined,
  });
  registrar = registrarResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

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
    threshold: { absolute_percentage: { percentage: "0.50" }},
    max_voting_period: { height: 1000 },
    max_voting_period_guardians: { height: 100 },
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
    fund_rotation: 10,
    // funding_goal: "50000000",
  });
  indexFund = fundResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${indexFund}`);

  // Anchor Vault - #1
  process.stdout.write("Instantiating Anchor Vault (#1) contract");
  const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    tax_per_block: "0.0000000259703196", // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #1",
    symbol: "apANC1",
    decimals: 6,
  });
  anchorVault1 = vaultResult1.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault1}`);

  // Anchor Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Anchor Vault (#2) contract");
  const vaultResult2 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    tax_per_block: "0.0000000259703196", // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #2",
    symbol: "apANC",
    decimals: 6,
  });
  anchorVault2 = vaultResult2.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault2}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault #1 & #2 in Registrar");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: { 
        vault_addr: anchorVault1,
        approved: true,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      vault_update_status: { 
        vault_addr: anchorVault2,
        approved: true,
      },
    })
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("Set default vault in Registrar (for newly created Endowments) as Anchor Vault #1");
  process.stdout.write("Update Registrar with the Address of the Index Fund contract");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        default_vault: anchorVault1,
        index_fund_contract: indexFund,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Step 4: Create Endowments via the Registrar contract
  // endowment #1
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  const charityResult1 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity1.key.accAddress,
        beneficiary: charity1.key.accAddress,
        name: "Test Endowment #1",
        description: "A wonderful charity endowment that aims to test all the things",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract1 = charityResult1.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract1}`);

  // endowment #2
  process.stdout.write("Charity Endowment #2 created from the Registrar by the AP Team");
  const charityResult2 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity2.key.accAddress,
        beneficiary: charity2.key.accAddress,
        name: "Test Endowment #2",
        description: "An even better endowment full of butterflies and rainbows",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract2 = charityResult2.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract2}`);

  // endowment #3
  process.stdout.write("Charity Endowment #3 created from the Registrar by the AP Team");
  const charityResult3 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity3.key.accAddress,
        beneficiary: charity3.key.accAddress,
        name: "Test Endowment #3",
        description: "Shady endowment that will never be approved",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract3 = charityResult3.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract3}`);

  // endowment #4
  process.stdout.write("Charity Endowment #4 created from the Registrar by the AP Team");
  const charityResult4 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity3.key.accAddress,
        beneficiary: charity3.key.accAddress,
        name: "Vibin' Endowment #4",
        description: "Global endowment that spreads good vibes",
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
      }
    }),
  ]);
  endowmentContract4 = charityResult4.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => { 
    return attribute.key == "contract_address"; 
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${endowmentContract4}`);

  // AP Team approves 3 of 4 newly created endowments
  process.stdout.write("AP Team approves 3 of 4 endowments");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract1,
        status: 1,
        beneficiary: undefined,
      }
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract2,
        status: 1,
        beneficiary: undefined,
      }
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract4,
        status: 1,
        beneficiary: undefined,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Step 5: Index Fund finals setup 
  // Create an initial "Fund" with the two charities created above
  process.stdout.write("Create two Funds with two endowments each");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: 1,
          name: "Test Fund",
          description: "My first test fund",
          members: [endowmentContract1, endowmentContract2],
        }
      }
    }),
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        fund: {
          id: 2,
          name: "Test Fund #2",
          description: "Another fund to test rotations",
          members: [endowmentContract1, endowmentContract4],
        }
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // Add confirmed TCA Members to the Index Fund SCs approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_tca_list: { new_list: [tca.key.accAddress] },
    }),
  ]);
  console.log(chalk.green(" Done!"));

  // // Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
  // process.stdout.write("Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract");
  // await sendTransaction(terra, apTeam, [
  //   new MsgExecuteContract(apTeam.key.accAddress, registrar, {
  //     update_owner: { new_owner: cw3ApTeam },
  //   }),
  //   new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
  //     update_owner: { new_owner: cw3ApTeam },
  //   }),
  //   new MsgExecuteContract(apTeam.key.accAddress, endowmentContract1, {
  //     update_owner: { new_owner: cw3ApTeam },
  //   }),
  //   new MsgExecuteContract(apTeam.key.accAddress, endowmentContract2, {
  //     update_owner: { new_owner: cw3ApTeam },
  //   }),
  //   new MsgExecuteContract(apTeam.key.accAddress, endowmentContract3, {
  //     update_owner: { new_owner: cw3ApTeam },
  //   }),
  //   // new MsgExecuteContract(apTeam.key.accAddress, anchorVault1, {
  //   //   update_owner: { new_owner: cw3ApTeam },
  //   // }),
  //   // new MsgExecuteContract(apTeam.key.accAddress, anchorVault2, {
  //   //   update_owner: { new_owner: cw3ApTeam },
  //   // }),
  // ]);
  // console.log(chalk.green(" Done!"));
}


//----------------------------------------------------------------------------------------
// TEST: AP Team Closes Endowment
//
// SCENARIO:
// AP Team Wallet needs close an endowment for a charity that is undergoing legal 
// proceedings in it's country of origin.
//
//----------------------------------------------------------------------------------------
export async function testClosingEndpoint(): Promise<void> {
  process.stdout.write("AP Team closes down endowment #3 - Sends to beneficiary");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract3,
        status: 3,
        beneficiary: apTeam.key.accAddress,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write("AP Team closes down endowment #4 - Sends to parent Index Fund");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract4,
        status: 3,
        beneficiary: undefined,
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
}


export async function testUpdatingRegistrarConfigs(): Promise<void> {
  process.stdout.write("AP Team updates Registrar Tax Rate");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        tax_rate: "0.2",
      }
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Add a new AP Team Member to the C4 AP Team Group
//
// SCENARIO:
// New AP Team Wallet needs to be added to the C4 Group. Done via a new proposal
// by an existing group member, approved with YES votes, and executed by any wallet.
//
//----------------------------------------------------------------------------------------

export async function testAddApTeamMemberToC4Group(): Promise<void> {
  process.stdout.write("Test - Propose and Execute adding a new member to AP Team C4 Group");

  // proposal to add new member
  const proposal = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, cw3ApTeam, {
      propose: {
        title: "New CW4 member",
        description: "New member for the CW4 AP Team Group. They are legit, I swear!",
        msgs: [
          { wasm: {
            execute: {
              contract_addr: cw4GrpApTeam,
              funds: [],
              msg: toEncodedBinary({
                update_members: {
                  add: [{ addr:apTeam3.key.accAddress, weight:1 }],
                  remove: [],
                }
              })
            }
          }
        }]
      }
    })
  ]);
  const proposal_id = proposal.logs[0].events.find((event) => {
      return event.type == "wasm";
    })?.attributes.find((attribute) => { 
      return attribute.key == "proposal_id"; 
    })?.value as string;
  // execute the proposal (anyone can do this for passed proposals)
  await expect(
    sendTransaction(terra, apTeam3, [
      new MsgExecuteContract(apTeam3.key.accAddress, cw3ApTeam, {
        execute: { proposal_id: parseInt(proposal_id) }
      })
    ])
  );
  
  console.log(chalk.green("Passed!"));
}

export async function testAddGuardiansToEndowment(): Promise<void> {
  process.stdout.write("Test - Endowment Owner Proposes and Executes adding 3 Guardians to their Endowment");

  // proposal to add new Guardians
  const proposal = await sendTransaction(terra, charity1, [
    new MsgExecuteContract(charity1.key.accAddress, cw3GuardianAngels, {
      propose_guardian_change: {
        endowment_addr: endowmentContract1,
        add: [charity3.key.accAddress, apTeam3.key.accAddress, charity2.key.accAddress],
        remove: [],
      }
    })
  ]);

  const proposal_id = parseInt(proposal.logs[0].events.find((event) => {
      return event.type == "wasm";
    })?.attributes.find((attribute) => { 
      return attribute.key == "proposal_id"; 
    })?.value as string);

  // execute the proposal (anyone can do this for passed proposals)
  await sendTransaction(terra, pleb, [
    new MsgExecuteContract(pleb.key.accAddress, cw3GuardianAngels, {
      execute: { proposal_id: proposal_id }
    })
  ]);
  
  console.log(chalk.green("Passed!"));
}

export async function testGuardiansChangeEndowmentOwner(): Promise<void> {
  process.stdout.write("Test - Endowment Owner loses wallet! :( Guardians Propose, vote and execute a change to new wallet");

  // proposal to add new Guardians
  const proposal = await sendTransaction(terra, charity2, [
    new MsgExecuteContract(charity2.key.accAddress, cw3GuardianAngels, {
      propose_owner_change: {
        endowment_addr: endowmentContract1,
        new_owner_addr: pleb.key.accAddress,
      }
    })
  ]);

  const proposal_id = parseInt(proposal.logs[0].events.find((event) => {
      return event.type == "wasm";
    })?.attributes.find((attribute) => { 
      return attribute.key == "proposal_id"; 
    })?.value as string);

  // Guardians vote on the open proposal until threshold reached
  await sendTransaction(terra, charity3, [
    new MsgExecuteContract(charity3.key.accAddress, cw3GuardianAngels, {
      vote_guardian: {
        proposal_id: proposal_id,
        vote: "yes"  
      }
    }),
    // execute the proposal (anyone can do this for passed proposals)
    new MsgExecuteContract(charity3.key.accAddress, cw3GuardianAngels, {
      execute: { proposal_id: proposal_id }
    }),
  ]);

  console.log(chalk.green("Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: Normal Donor cannot send funds to the Index Fund 
//
// SCENARIO:
// Normal user sends UST funds to an Index Fund SC fund to have it split 
// up amonst the fund's charity members. 
//
//----------------------------------------------------------------------------------------

export async function testDonorSendsToIndexFund(): Promise<void> {
  process.stdout.write("Test - Donor (normal pleb) cannot send a UST donation to an Index Fund fund");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(pleb.key.accAddress, indexFund,
        {
          deposit: {
            fund_id: 1,
            split: undefined,
          },
        },
        { uusd: "4200000", }
      ),
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate
  console.log(chalk.green("Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: Cannot send funds to an Endowment that is not approved for deposits 
//
// SCENARIO:
// If an Endowment has not been approved by the AP Team, all deposits should be rejected
//
//----------------------------------------------------------------------------------------

export async function testRejectUnapprovedDonations(): Promise<void> {
  process.stdout.write("Test - Donors cannot send donation to unapproved Accounts");

  await expect(
    sendTransaction(terra, pleb, [
      new MsgExecuteContract(
        pleb.key.accAddress,
        endowmentContract3,
        {
          deposit: {
            locked_percentage: "0.8",
            liquid_percentage: "0.2",
          },
        },
        { uusd: "4200000", }
      ),
    ])
  ).to.be.rejectedWith("Unauthorized"); // for MVP normal users cannot donate
  console.log(chalk.green("Passed!"));
}


//----------------------------------------------------------------------------------------
// TEST: TCA Member can send donations to the Index Fund 
//
// SCENARIO:
// TCA Member sends UST funds to an Index Fund SC fund to have it split 
// up amonst the active fund's charity members. 
//
//----------------------------------------------------------------------------------------

export async function testTcaMemberSendsToIndexFund(): Promise<void> {
  process.stdout.write("Test - TCA Member can send a UST donation to an Index Fund");
  await expect(
    sendTransaction(terra, tca, [
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: undefined, split: undefined, }, },
        { uusd: "30000000", }
      ),
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: 1, split: undefined, }, },
        { uusd: "40000000", }
      ),
      new MsgExecuteContract(
        tca.key.accAddress,
        indexFund,
        { deposit: { fund_id: 1, split: "0.76", }, },
        { uusd: "40000000", }
      ),
    ])
  )
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team and trigger harvesting of Accounts for a Vault(s)
//
// SCENARIO:
// AP team needs to send a message to a Vault to trigger a rebalance of Endowment funds, 
// moving money from their Locked to Liquid & taking a small tax of DP Tokens as well.
//
//----------------------------------------------------------------------------------------
export async function testAngelTeamCanTriggerVaultsHarvest(): Promise<void> {
  process.stdout.write("Test - AP Team can trigger harvest of all Vaults (Locked to Liquid Account)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, registrar, {
        harvest: {}
      })
    ])
  ).to.be.rejectedWith("Unauthorized");

  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        harvest: {}
      })
    ])
  );

  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Charity Beneficiary can withdraw from available balance in their Accounts
//
// SCENARIO:
// Charity beneficiary can draw down on the available Liquid Account balance and should
// not be able to touch the Locked Account's balance.
//
//----------------------------------------------------------------------------------------
export async function testBeneficiaryCanWithdrawFromLiquid(): Promise<void> {
  process.stdout.write("Test - Beneficiary can withdraw from the Endowment availalble amount (liquid)");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "500000", liquid: "1000000"},
            {vault: anchorVault2, locked: "500000", liquid: "1000000"}
          ]
        }
      })
    ])
  ).to.be.rejectedWith("Cannot withdraw from Locked balances");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        withdraw: {
          sources: [
            {vault: anchorVault1, locked: "0", liquid: "5000000"},
          ]
        }
      })
    ])
  );

  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Charity Owner can rebalance their portfolio/update the Accounts' strategy
//
// SCENARIO:
// Charity Owner can trigger a rebalance of their Accounts, which should: 
// 1) redeem all invested funds from Vaults to the Accounts
// 2) reinvest all redeemed funds, according the accounts' strategy
//
//----------------------------------------------------------------------------------------

export async function testCharityCanUpdateStrategies(): Promise<void> {
  process.stdout.write("Test - Charity can update their Endowment's strategies");

  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, endowmentContract1, {
        update_strategies: {
          strategies: [
            {vault: anchorVault1, locked_percentage: "0.5", liquid_percentage: "0.5"},
            {vault: anchorVault2, locked_percentage: "0.5", liquid_percentage: "0.5"}
          ]
        }
      })
    ])
  );
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team can trigger migration of all Account SC Endowments from Registrar
//----------------------------------------------------------------------------------------

export async function testMigrateAllAccounts(): Promise<void> {
  process.stdout.write("Test - AP Team can trigger migration of all Account SC Endowments from Registrar");
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        migrate_accounts: {},
      }),
    ])
  );
  console.log(chalk.green("Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryRegistrarConfig(): Promise<void> {
  process.stdout.write("Test - Query Registrar config and get proper result");
  const result: any = await terra.wasm.contractQuery(registrar, {
    config: {},
  });

  // expect(result.accounts_code_id).to.equal(accountsCodeId);
  expect(result.treasury).to.equal(apTeam.key.accAddress);
  expect(result.default_vault).to.equal(anchorVault1);
  expect(result.index_fund).to.equal(indexFund);
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentList(): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    endowment_list: {},
  });

  expect(result.endowments.length).to.equal(3);
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultList(): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedVaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_list: {},
  });

  // expect(result.vaults.length).to.equal(1);
  // expect(result.vaults[0].address).to.equal(anchorVault1);
  expect(result.vaults[0].input_denom).to.equal('uusd');
  expect(result.vaults[0].yield_token).to.equal(yieldToken);
  expect(result.vaults[0].approved).to.equal(true);

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

  // expect(result.vaults.length).to.equal(1);
  console.log(result);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVault(): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault: {
      vault_addr: anchorVault1,
    },
  });

  expect(result.vault.address).to.equal(anchorVault1);
  expect(result.vault.input_denom).to.equal('uusd');
  expect(result.vault.yield_token).to.equal(yieldToken);
  expect(result.vault.approved).to.equal(true);
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsBalance(addr: string): Promise<void> {
  process.stdout.write("Test - Query Accounts Balance");
  const result: any = await terra.wasm.contractQuery(addr, {
    balance: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsConfig(): Promise<void> {
  process.stdout.write("Test - Query Accounts Config");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    config: {},
  });

  expect(result.registrar_contract).to.equal(registrar);
  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultConfig(addr: string): Promise<void> {
  process.stdout.write("Test - Query Vault Config");
  const result: any = await terra.wasm.contractQuery(addr, {
    vault_config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryAccountsEndowment(): Promise<void> {
  process.stdout.write("Test - Query Accounts Endowment");
  const result: any = await terra.wasm.contractQuery(endowmentContract1, {
    endowment: {},
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
