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
let cw4GrpApTeam: string;
let cw3ApTeam: string;
let indexFund: string;
let donationMatchCharities: string;
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
    fund_rotation: number | undefined;
    is_localjuno: boolean;
    harvest_to_liquid: string;
    tax_per_block: string;
    funding_goal: string | undefined;
    fund_member_limit: number | undefined;
    accepted_tokens: any | undefined;
    charity_cw3_threshold_abs_perc: string,
    charity_cw3_max_voting_period: number,
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
  // if (!config.is_localjuno) {
  //   await createVaults(config.harvest_to_liquid, config.tax_per_block);
  // }
  await turnOverApTeamMultisig(config.is_localjuno);
  await createEndowments(
    config.charity_cw3_threshold_abs_perc,
    config.charity_cw3_max_voting_period,
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

  process.stdout.write("Uploading Standard CW3 MultiSig Wasm");
  const cw3MultiSig = await storeCode(juno, apTeamAddr, `${wasm_path.core}/cw3_multisig.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSig}`);

  process.stdout.write("Uploading Endowment CW3 MultiSig Wasm");
  const cw3MultiSigEndowment = await storeCode(juno, apTeamAddr, `${wasm_path.core}/endowment_cw3_multisig.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw3MultiSigEndowment}`);

  process.stdout.write("Uploading Endowment SubDAO Wasm");
  const subdao = await storeCode(juno, apTeamAddr, `${wasm_path.core}/subdao.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdao}`);

  process.stdout.write("Uploading Endowment SubDAO Token Wasm");
  const subdaoToken = await storeCode(juno, apTeamAddr, `${wasm_path.core}/subdao_token.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${subdaoToken}`);

  process.stdout.write("Uploading Endowment SubDAO Donation Matching Wasm");
  const subdaoDonationMatch = await storeCode(juno, apTeamAddr, `${wasm_path.core}/donation_match.wasm`);
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
      accounts_code_id: accountsCodeId,
      treasury: treasury_address,
      default_vault: apTeamAddr, // Fake vault address from apTeam
      split_to_liquid: undefined,
      accepted_tokens: {
        native: ['ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034', 'ujunox'],
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
    cw3MultiSig,
    {
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
  
  // Charities Donation Matching
  process.stdout.write("Instantiating Charities Donation Matching contract");
  const charityDonationMatchResult = await instantiateContract(juno, apTeamAddr, apTeamAddr, subdaoDonationMatch, {
    registrar_contract: registrar,
    reserve_token: apTeamAddr, // FAKE! Need to fix.
    lp_pair: apTeamAddr, // FAKE! Need to fix.
      });
  donationMatchCharities = charityDonationMatchResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${donationMatchCharities}`);
  
  process.stdout.write("Update Registrar's config with various wasm codes & contracts");
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_config: {
      accounts_code_id: accountsCodeId,
      index_fund_contract: indexFund,
      cw3_code: cw3MultiSigEndowment,
      cw4_code: cw4Group,
      halo_token: apTeamAddr, // Fake halo token addr: Need to be handled
      halo_token_lp_contract: apTeamAddr, // Fake halo token LP addr: Need to be handled
      subdao_gov_code: subdao,
      subdao_token_code: subdaoToken,
      donation_match_code: subdaoDonationMatch,
      donation_match_charites_contract: donationMatchCharities,
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
  const charityResult1 = await sendTransaction(juno, apTeamAddr, registrar, {
    create_endowment: {
      owner: charity1_wallet,
      withdraw_before_maturity: false,
      maturity_time: 300,
      split_max: undefined,
      split_min: undefined,
      split_default: undefined,
      cw4_members: [{ addr: charity1_wallet, weight: 1 }],
      cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_max_voting_period: charity_cw3_max_voting_period,
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
      kyc_donors_only: false,
      whitelisted_beneficiaries: [charity1_wallet], 
      whitelisted_contributors: [],
      dao: {
        quorum: "0.2",
        threshold: "0.5",
        voting_period: 1000000,
        timelock_period: 1000000,
        expiration_period: 1000000,
        proposal_deposit: "1000000",
        snapshot_period: 1000,
        token: {
          bonding_curve: {
            curve_type: {
              square_root: {
                slope: "19307000",
                power: "428571429",
                scale: 9,
              }
            },
            name: "AP Endowment DAO Token",
            symbol: "APEDT",
            decimals: 6,
            reserve_decimals: 6,
            reserve_denom: "ujunox",
            unbonding_period: 1, 
          }
        }
      },
      donation_match: undefined,
      earnings_fee: undefined,
      deposit_fee: undefined,
      withdraw_fee: undefined,
      aum_fee: undefined,
      settings_controller: undefined,
      parent: false,
    }
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
      withdraw_before_maturity: false,
      maturity_time: 300,
      split_max: undefined,
      split_min: undefined,
      split_default: undefined,
      cw4_members: [{ addr: charity2_wallet, weight: 1 }],
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_max_voting_period,
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
      kyc_donors_only: false,
      whitelisted_beneficiaries: [charity2_wallet], 
      whitelisted_contributors: [],
      dao: undefined,
      donation_match: undefined,
      earnings_fee: undefined,
      deposit_fee: undefined,
      withdraw_fee: undefined,
      aum_fee: undefined,
      settings_controller: undefined,
      parent: false,
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
      withdraw_before_maturity: false,
      maturity_time: 300,
      split_max: undefined,
      split_min: undefined,
      split_default: undefined,
      cw4_members: [{ addr: charity3_wallet, weight: 1 }],
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_max_voting_period,
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
      kyc_donors_only: false,
      whitelisted_beneficiaries: [charity1_wallet], 
      whitelisted_contributors: [],
      dao: undefined,
      donation_match: undefined,
      earnings_fee: undefined,
      deposit_fee: undefined,
      withdraw_fee: undefined,
      aum_fee: undefined,
      settings_controller: undefined,
      parent: false,
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
      withdraw_before_maturity: false,
      maturity_time: 300,
      split_max: undefined,
      split_min: undefined,
      split_default: undefined,
      cw4_members: [{ addr: charity3_wallet, weight: 1 }],
      cw3_multisig_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
      cw3_multisig_max_vote_period: charity_cw3_max_voting_period,
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
      kyc_donors_only: false,
      whitelisted_beneficiaries: [charity1_wallet], 
      whitelisted_contributors: [],
      dao: undefined,
      donation_match: undefined,
      earnings_fee: undefined,
      deposit_fee: undefined,
      withdraw_fee: undefined,
      aum_fee: undefined,
      settings_controller: undefined,
      parent: false,
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
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    update_endowment_status: {
      endowment_addr: endowmentContract1,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    update_endowment_status: {
      endowment_addr: endowmentContract2,
      status: 1,
      beneficiary: undefined,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
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
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
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
    await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, indexFund, {
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
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_update_status: {
      vault_addr: vault1,
      approved: true,
    }
  });
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    vault_update_status: {
      vault_addr: vault2,
      approved: true,
    }
  });
  console.log(chalk.green(" Done!"));

  process.stdout.write("Set default vault in Registrar as Anchor Vault");
  await sendMessageViaCw3Proposal(juno, apTeamAddr, cw3ApTeam, registrar, {
    update_config: { default_vault: vault1 }
  });
  console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(is_localjuno: boolean): Promise<void> {
  process.stdout.write(
    "Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract"
  );
  process.stdout.write(chalk.yellow("\n> Turning over Registrar"));
  await sendTransaction(juno, apTeamAddr, registrar, {
    update_owner: { new_owner: cw3ApTeam }
  });
  console.log(chalk.green(" Done!"));
  
  process.stdout.write(chalk.yellow("\n> Turning over Index Fund"));
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
