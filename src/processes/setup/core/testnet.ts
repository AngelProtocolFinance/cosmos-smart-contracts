/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { sendTransaction, storeCode, instantiateContract, getWalletAddress } from "../../../utils/helpers";
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
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let indexFund: string;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;
let endowmentContract4: string;
let vault1: string;
let vault2: string;

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
    max_voting_period_guardians_height: number;
    fund_rotation: number | undefined;
    turnover_to_multisig: boolean;
    is_localjuno: boolean;
    harvest_to_liquid: string;
    tax_per_block: string;
    funding_goal: string | undefined;
    charity_cw3_multisig_threshold_abs_perc: string,
    charity_cw3_multisig_max_voting_period: number,
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
    config.max_voting_period_guardians_height,
    config.fund_rotation,
    config.funding_goal,
    config.is_localjuno,
  );
  // if (!config.is_localjuno) {
  //   await createVaults(config.harvest_to_liquid, config.tax_per_block);
  // }
  if (config.turnover_to_multisig) {
    await turnOverApTeamMultisig(config.is_localjuno);
  }
  await createEndowments(
    config.charity_cw3_multisig_threshold_abs_perc,
    config.charity_cw3_multisig_max_voting_period,
  );
  await approveEndowments();
  await createIndexFunds();
}

async function setup(
  tax_rate: string,
  treasury_address: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  max_voting_period_guardians_height: number,
  fund_rotation: number | undefined,
  funding_goal: string | undefined,
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

  process.stdout.write("Uploading CW3 MultiSig Wasm");
  const cw3MultiSig = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_multisig.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSig}`);

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
      accounts_code_id: accountsCodeId,
      treasury: treasury_address,
      default_vault: undefined,
      split_to_liquid: undefined,
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujuno'],
        cw20: [],
      }
    }
  );
  registrar = registrarResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${registrar}`);

  // Index Fund
  process.stdout.write("Instantiating Index Fund contract");
  const fundResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, fundCodeId, {
    registrar_contract: registrar,
    fund_rotation: fund_rotation,
    funding_goal: funding_goal,
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
    cw3MultiSig,
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
  await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
    add_hook: { addr: cw3ApTeam },
  });
  await sendTransaction(juno, apTeamAddr, cw4GrpApTeam, {
    update_admin: { admin: cw3ApTeam },
  });
  console.log(chalk.green(" Done!"));

  // // Add confirmed TCA Members to the Index Fund SCs approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  let tca_wallet = await getWalletAddress(tca);
  await sendTransaction(juno, apTeamAddr, indexFund, {
    update_alliance_member_list: {
      address: tca_wallet,
      member: {
        name: "TCA Member",
        logo: undefined,
        website: "https://angelprotocol.io/app",
      },
      action: "add",
    },
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write("Update Registrar's config Index Fund, CW3_code_Id, CW4_code_Id");
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_config: {
      index_fund_contract: indexFund,
      cw3_code: cw3MultiSig,
      cw4_code: cw4Group,
    },
  });
  console.log(chalk.green(" Done!"));
}

// Step 4: Create Endowments via the Registrar contract
async function createEndowments(
  charity_cw3_multisig_threshold_abs_perc: string,
  charity_cw3_multisig_max_voting_period: number,
): Promise<void> {
  // endowment #1
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  let charity1_wallet = await getWalletAddress(charity1);
  const charityResult1 = await sendTransaction(juno, apTeamAddr, registrar, {
    create_endowment: {
      owner: charity1_wallet,
      beneficiary: charity1_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #1",
        overview: "A wonderful charity endowment that aims to test all the things",
        un_sdg: 1,
        tier: 3,
        logo: "logo1",
        image: "image1",
        url: undefined,
        registration_number: undefined,
        country_city_origin: undefined,
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
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_multisig_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_multisig_max_voting_period,
    },
  });
  endowmentContract1 = charityResult1.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract1}`
  );

  // endowment #2
  process.stdout.write("Charity Endowment #2 created from the Registrar by the AP Team");
  let charity2_wallet = await getWalletAddress(charity2);
  const charityResult2 = await sendTransaction(juno, apTeamAddr, registrar, {
    create_endowment: {
      owner: charity2_wallet,
      beneficiary: charity2_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #2",
        overview: "An even better endowment full of butterflies and rainbows",
        un_sdg: 3,
        tier: 2,
        logo: "logo2",
        image: "image2",
        url: undefined,
        registration_number: undefined,
        country_city_origin: undefined,
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
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_multisig_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_multisig_max_voting_period,
    },
  });
  endowmentContract2 = charityResult2.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract2}`
  );

  // endowment #3
  process.stdout.write("Charity Endowment #3 created from the Registrar by the AP Team");
  let charity3_wallet = await getWalletAddress(charity3);
  const charityResult3 = await sendTransaction(juno, apTeamAddr, registrar, {
    create_endowment: {
      owner: charity3_wallet,
      beneficiary: charity3_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Test Endowment #3",
        overview: "Shady endowment that will never be approved",
        un_sdg: 2,
        tier: 1,
        logo: "logo3",
        image: "image3",
        url: undefined,
        registration_number: undefined,
        country_city_origin: undefined,
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
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_multisig_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_multisig_max_voting_period,
    },
  });
  endowmentContract3 = charityResult3.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract3}`
  );

  // endowment #4
  process.stdout.write("Charity Endowment #4 created from the Registrar by the AP Team");
  const charityResult4 = await sendTransaction(juno, apTeamAddr, registrar, {
    create_endowment: {
      owner: charity3_wallet,
      beneficiary: charity3_wallet,
      withdraw_before_maturity: false,
      maturity_time: undefined,
      maturity_height: undefined,
      profile: {
        name: "Vibin' Endowment #4",
        overview: "Global endowment that spreads good vibes",
        un_sdg: 1,
        tier: 3,
        logo: "logo4",
        image: "image4",
        url: undefined,
        registration_number: undefined,
        country_city_origin: undefined,
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
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_multisig_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_multisig_max_voting_period,
    }
  });
  endowmentContract4 = charityResult4.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract4}`
  );
}

async function approveEndowments(): Promise<void> {
  // AP Team approves 3 of 4 newly created endowments
  process.stdout.write("AP Team approves 3 of 4 endowments");
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_endowment_status: {
      endowment_addr: endowmentContract1,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_endowment_status: {
      endowment_addr: endowmentContract2,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_endowment_status: {
      endowment_addr: endowmentContract4,
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
  await sendTransaction(juno, apTeamAddr, indexFund, {
      create_fund: {
        name: "Test Fund",
        description: "My first test fund",
        members: [endowmentContract1, endowmentContract2],
        rotating_fund: true,
        split_to_liquid: undefined,
        expiry_time: undefined,
        expiry_height: undefined,
      }
    });
    await sendTransaction(juno, apTeamAddr, indexFund, {
      create_fund: {
        name: "Test Fund #2",
        description: "Another fund to test rotations",
        members: [endowmentContract1, endowmentContract4],
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
  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(juno, apTeamAddr, `${wasm_path.core}/anchor.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // Anchor Vault - #1
  process.stdout.write("Instantiating Anchor Vault (#1) contract");
  const vaultResult1 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #1",
    symbol: "apANC1",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault1 = vaultResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vault1}`);

  // Anchor Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Anchor Vault (#2) contract");
  const vaultResult2 = await instantiateContract(juno, apTeamAddr, apTeamAddr, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #2",
    symbol: "apANC2",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  vault2 = vaultResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${vault2}`);

  // Step 3. AP team must approve the new anchor vault in registrar & make it the default vault
  process.stdout.write("Approving Anchor Vault #1 & #2 in Registrar");
  await sendTransaction(juno, apTeamAddr, registrar, {
    vault_update_status: {
      vault_addr: vault1,
      approved: true,
    }
  });
  await sendTransaction(juno, apTeamAddr, registrar, {
    vault_update_status: {
      vault_addr: vault2,
      approved: true,
    }
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write("Set default vault in Registrar as Anchor Vault");
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_config: { default_vault: vault1 }
  });
  console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(is_localjuno: boolean): Promise<void> {
  process.stdout.write(
    "Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract"
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
  
  // if (!is_localjuno) {
  //   await sendTransaction(juno, apTeamAddr, vault1, {
  //     update_owner: { new_owner: cw3ApTeam }
  //   });
  //   process.stdout.write(chalk.yellow("\n- Turning over Vault 1"));
  //   await sendTransaction(juno, apTeamAddr, vault2, {
  //     update_owner: { new_owner: cw3ApTeam }
  //   });
  //   process.stdout.write(chalk.yellow("\n- Turning over Vault 2"));
  // }
  console.log(chalk.green(" Done!"));
}
