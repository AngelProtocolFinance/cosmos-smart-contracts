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
let apTeam2: DirectSecp256k1HdWallet;
let apTreasury: DirectSecp256k1HdWallet;
let charity1: DirectSecp256k1HdWallet;
let charity2: DirectSecp256k1HdWallet;
let charity3: DirectSecp256k1HdWallet;
let tca: DirectSecp256k1HdWallet;

// wallet addresses
let apTeamAddr: string;
let apTeam2Addr: string;
let apTreasuryAddr: string;

let registrar: string;
let accounts: string;
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let cw4GrpReviewTeam: string;
let cw3ReviewTeam: string;
let indexFund: string;
let vault1: string;
let vault2: string;

let endow_1_id: number;
let endow_2_id: number;
let endow_3_id: number;
let endow_4_id: number;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------

export async function setupCore(
  _juno: SigningCosmWasmClient,
  wallets: {
    apTeam: DirectSecp256k1HdWallet;
    apTeam2: DirectSecp256k1HdWallet;
    apTeam3: DirectSecp256k1HdWallet;
    apTreasury: DirectSecp256k1HdWallet;
    charity1: DirectSecp256k1HdWallet;
    charity2: DirectSecp256k1HdWallet;
    charity3: DirectSecp256k1HdWallet;
    tca: DirectSecp256k1HdWallet;
  },
  config: {
    tax_rate: string;
    threshold_absolute_percentage: string;
    max_voting_period_height: number;
    fund_rotation: number | undefined;
    is_localjuno: boolean;
    harvest_to_liquid: string;
    tax_per_block: string;
    funding_goal: string | undefined;
    fund_member_limit: number | undefined;
    charity_cw3_threshold_abs_perc: string,
    charity_cw3_max_voting_period: number,
    accepted_tokens: any | undefined;
  }
): Promise<void> {
  juno = _juno;
  apTeam = wallets.apTeam;
  apTeam2 = wallets.apTeam2;
  apTreasury = wallets.apTreasury;
  charity1 = wallets.charity1;
  charity2 = wallets.charity2;
  charity3 = wallets.charity3;
  tca = wallets.tca;

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
    config.is_localjuno,
  );
  await turnOverApTeamMultisig();
  if (!config.is_localjuno) {
    await createVaults(config.harvest_to_liquid, config.tax_per_block);
  }
  await createEndowments(
    config.threshold_absolute_percentage,
    config.max_voting_period_height,
  );
  await approveEndowments();
  await createIndexFunds();
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
  is_localjuno: boolean
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each....
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/registrar.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

  process.stdout.write("Uploading Index Fund Wasm");
  const fundCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/index_fund.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${fundCodeId}`);

  process.stdout.write("Uploading Accounts Wasm");
  const accountsCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/accounts.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${accountsCodeId}`);

  process.stdout.write("Uploading CW4 Group Wasm");
  const cw4Group = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw4_group.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw4Group}`);

  process.stdout.write("Uploading AP Team CW3 MultiSig Wasm");
  const cw3MultiSigApTeam = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_apteam.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigApTeam}`);

  process.stdout.write("Uploading Generic CW3 MultiSig Wasm");
  const cw3MultiSigGeneric = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_generic.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigGeneric}`);

  process.stdout.write("Uploading Endowment CW3 MultiSig Wasm");
  const cw3MultiSigEndowment = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_endowment.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigEndowment}`);

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
      default_vault: apTeamAddr, // Fake vault address from apTeam
      split_to_liquid: undefined,
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
  const cw4GrpApTeamResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, cw4Group, {
    admin: apTeamAddr,
    members: [
      { addr: apTeamAddr, weight: 1 },
      { addr: apTeam2Addr, weight: 1 },
    ],
  });
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
      // registrar_contract: registrar,
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

  process.stdout.write("Instantiating the Accounts contract");
  const accountsResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    accountsCodeId,
    {
      owner_sc: apTeamAddr,
      registrar_contract: registrar,
    }
  );
  accounts = accountsResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${accounts}`);

  // CW4 Review Team Group
  process.stdout.write("Instantiating CW4 Review Team Group contract");
  const cw4GrpReviewTeamResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, cw4Group, {
    admin: apTeamAddr,
    members: [
      { addr: apTeamAddr, weight: 1 },
    ],
  });
  cw4GrpReviewTeam = cw4GrpReviewTeamResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${cw4GrpReviewTeam}`);

  // CW3 Review Team MultiSig
  process.stdout.write("Instantiating CW3 Review Team MultiSig contract");
  const cw3ReviewTeamResult = await instantiateContract(
    juno,
    apTeamAddr,
    apTeamAddr,
    cw3MultiSigGeneric,
    {
      registrar_contract: registrar,
      group_addr: cw4GrpReviewTeam,
      threshold: { absolute_percentage: { percentage: threshold_absolute_percentage } },
      max_voting_period: { height: max_voting_period_height },
      // registrar_contract: registrar,
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
      halo_token: apTeamAddr, // Fake halo token addr: Need to be handled
      halo_token_lp_contract: apTeamAddr, // Fake halo token LP addr: Need to be handled
    },
  });
  console.log(chalk.green(" Done!"));
}

// Step 4: Create Endowments via the Registrar contract
async function createEndowments(
  charity_cw3_threshold_abs_perc: string,
  charity_cw3_max_voting_period: number,
): Promise<void> {
  // endowment #1
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  let charity1_wallet = await getWalletAddress(charity1);
  const charityResult1 = await sendTransaction(juno, apTeamAddr, accounts, {
    create_endowment: {
      owner: charity1_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #1",
        overview: "A wonderful charity endowment that aims to test all the things",
        categories: { sdgs:[1], general: [] },
        tier: 3,
        logo: "logo1",
        image: "image1",
        url: undefined,
        registration_number: undefined,
        country_of_origin: undefined,
        street_address: undefined,
        contact_email: undefined,
        social_media_urls: {
          facebook: undefined,
          twitter: undefined,
          linkedin: undefined,
        },
        number_of_employees: undefined,
        average_annual_budget: undefined,
        annual_revenue: undefined,
        charity_navigator_rating: undefined,
        endow_type: "Charity",
      },
      cw4_members: [{ addr: charity1_wallet, weight: 1 }],
      kyc_donors_only: false,
      cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_max_voting_period: charity_cw3_max_voting_period,
    },
  });
  endow_1_id = parseInt(charityResult1.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Endowment_ID")}=${endow_1_id}`
  );

  // endowment #2
  process.stdout.write("Charity Endowment #2 created from the Registrar by the AP Team");
  let charity2_wallet = await getWalletAddress(charity2);
  const charityResult2 = await sendTransaction(juno, apTeamAddr, accounts, {
    create_endowment: {
      owner: charity2_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #2",
        overview: "An even better endowment full of butterflies and rainbows",
        categories: { sdgs:[3], general: [] },
        tier: 2,
        logo: "logo2",
        image: "image2",
        url: undefined,
        registration_number: undefined,
        country_of_origin: undefined,
        street_address: undefined,
        contact_email: undefined,
        social_media_urls: {
          facebook: undefined,
          twitter: undefined,
          linkedin: undefined,
        },
        number_of_employees: undefined,
        average_annual_budget: undefined,
        annual_revenue: undefined,
        charity_navigator_rating: undefined,
        endow_type: "Charity",
      },
      cw4_members: [{ addr: charity2_wallet, weight: 1 }],
      kyc_donors_only: false,
      cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_max_voting_period: charity_cw3_max_voting_period,
    },
  });
  endow_2_id = parseInt(charityResult2.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Endowment_ID")}=${endow_2_id}`
  );

  // endowment #3
  process.stdout.write("Charity Endowment #3 created from the Registrar by the AP Team");
  let charity3_wallet = await getWalletAddress(charity3);
  const charityResult3 = await sendTransaction(juno, apTeamAddr, accounts, {
    create_endowment: {
      owner: charity3_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #3",
        overview: "Shady endowment that will never be approved",
        categories: { sdgs:[2], general: [] },
        tier: 1,
        logo: "logo3",
        image: "image3",
        url: undefined,
        registration_number: undefined,
        country_of_origin: undefined,
        street_address: undefined,
        contact_email: undefined,
        social_media_urls: {
          facebook: undefined,
          twitter: undefined,
          linkedin: undefined,
        },
        number_of_employees: undefined,
        average_annual_budget: undefined,
        annual_revenue: undefined,
        charity_navigator_rating: undefined,
        endow_type: "Charity",
      },
      cw4_members: [{ addr: charity3_wallet, weight: 1 }],
      kyc_donors_only: false,
      cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_max_voting_period: charity_cw3_max_voting_period,
    },
  });
  endow_3_id = parseInt(charityResult3.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Endowment_ID")}=${endow_3_id}`
  );

  // endowment #4
  process.stdout.write("Charity Endowment #4 created from the Registrar by the AP Team");
  const charityResult4 = await sendTransaction(juno, apTeamAddr, accounts, {
    create_endowment: {
      owner: charity3_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Vibin' Endowment #4",
        overview: "Global endowment that spreads good vibes",
        categories: { sdgs:[1], general: [] },
        tier: 3,
        logo: "logo4",
        image: "image4",
        url: undefined,
        registration_number: undefined,
        country_of_origin: undefined,
        street_address: undefined,
        contact_email: undefined,
        social_media_urls: {
          facebook: undefined,
          twitter: undefined,
          linkedin: undefined,
        },
        number_of_employees: undefined,
        average_annual_budget: undefined,
        annual_revenue: undefined,
        charity_navigator_rating: undefined,
        endow_type: "Charity",
      },
      cw4_members: [{ addr: charity3_wallet, weight: 1 }],
      kyc_donors_only: false,
      cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_max_voting_period: charity_cw3_max_voting_period,
    }
  });
  endow_4_id = parseInt(charityResult4.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "endow_id";
    })?.value as string);
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Endowment_ID")}=${endow_4_id}`
  );
}

async function approveEndowments(): Promise<void> {
  // AP Team approves 3 of 4 newly created endowments
  process.stdout.write("AP Team approves 3 of 4 endowments");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ReviewTeam, accounts, {
    update_endowment_status: {
      endowment_id: endow_1_id,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ReviewTeam, accounts, {
    update_endowment_status: {
      endowment_id: endow_2_id,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ReviewTeam, accounts, {
    update_endowment_status: {
      endowment_id: endow_4_id,
      status: 1,
      beneficiary: undefined,
    }
  });
  console.log(chalk.green(" Done!"));
}

// Step 5: Index Fund finals setup
async function createIndexFunds(): Promise<void> {
  // Create an initial "Fund" with the two charities created above
  process.stdout.write("Create two Funds with two endowments each");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
      create_fund: {
        name: "Test Fund",
        description: "My first test fund",
        members: [endow_1_id, endow_2_id],
        rotating_fund: true,
        split_to_liquid: undefined,
        expiry_time: undefined,
        expiry_height: undefined,
      }
    });
    await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
      create_fund: {
        name: "Test Fund #2",
        description: "Another fund to test rotations",
        members: [endow_1_id, endow_4_id],
        rotating_fund: true,
        split_to_liquid: undefined,
        expiry_time: undefined,
        expiry_height: undefined,
      }
   });
   console.log(chalk.green(" Done!"));
}

async function createVaults(
  harvest_to_liquid: string,
  tax_per_block: string
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const vaultCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.mock_vault}/mock_vault.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // Anchor Vault - #1
  process.stdout.write("Instantiating Vault (#1) contract");
  const vaultResult1 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    input_denom: "ujunox", // testnet placeholder
    yield_token: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #1",
    symbol: "apANC1",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault1 = vaultResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vault1}`);

  // Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Vault (#2) contract");
  const vaultResult2 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    input_denom: "ujunox", // testnet placeholder
    yield_token: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - #2",
    symbol: "apANC2",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault2 = vaultResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vault2}`);

  // Step 3. AP team must add & approve the new vaults in registrar & make #1 the default vault
  process.stdout.write("Add Vault #1 & #2 in Registrar");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault1,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `locked`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault1,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `liquid`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault2,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `locked`,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_add: {
      network: "juno-1",
      vault_addr: vault2,
      input_denom: "ujunox",
      yield_token: registrar,
      restricted_from: [],
      acct_type: `liquid`,
    }
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
    update_owner: { new_owner: cw3ApTeam }
  });
  console.log(chalk.green(" Done!"));
  
  process.stdout.write(chalk.yellow("- Turning over Index Fund"));
  await sendTransaction(juno, apTeamAddr, indexFund, {
    update_owner: { new_owner: cw3ApTeam }
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write(chalk.yellow("- Turning over Accounts"));
  await sendTransaction(juno, apTeamAddr, accounts, {
    update_owner: { new_owner: cw3ApTeam }
  });
  console.log(chalk.green(" Done!"));
}
