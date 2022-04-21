/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, storeCode, instantiateContract } from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { NONAME } from "dns";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let terra: LocalTerra | LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let charity1: Wallet;
let charity2: Wallet;
let charity3: Wallet;
let tca: Wallet;

let registrar: string;
let cw4GrpOwners: string;
let cw4GrpApTeam: string;
let cw3GuardianAngels: string;
let cw3ApTeam: string;
let indexFund: string;
let anchorVault1: string;
let anchorVault2: string;
let anchorMoneyMarket: string | undefined;
let endowmentContract1: string;
let endowmentContract2: string;
let endowmentContract3: string;
let endowmentContract4: string;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalTerra and TestNet
// -------------------------------------------------------------------------------------

export async function setupCore(
  _terra: LocalTerra | LCDClient,
  _anchorMoneyMarket: string | undefined,
  treasury_address: string,
  wallets: {
    apTeam: Wallet;
    apTeam2: Wallet;
    apTeam3: Wallet;
    charity1: Wallet;
    charity2: Wallet;
    charity3: Wallet;
    tca: Wallet;
  },
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
  terra = _terra;
  apTeam = wallets.apTeam;
  apTeam2 = wallets.apTeam2;
  charity1 = wallets.charity1;
  charity2 = wallets.charity2;
  charity3 = wallets.charity3;
  tca = wallets.tca;

  anchorMoneyMarket = _anchorMoneyMarket;

  await setup(
    treasury_address,
    config.tax_rate,
    config.threshold_absolute_percentage,
    config.max_voting_period_height,
    config.max_voting_period_guardians_height,
    config.fund_rotation,
    config.funding_goal,
    config.is_localterra
  );
  if (!config.is_localterra && anchorMoneyMarket) {
    await createVaults(config.harvest_to_liquid, config.tax_per_block);
  }
  await createEndowments();
  await approveEndowments();
  await createIndexFunds();
  if (config.turnover_to_multisig) {
    await turnOverApTeamMultisig(config.is_localterra);
  }
}

async function setup(
  treasury_address: string,
  tax_rate: string,
  threshold_absolute_percentage: string,
  max_voting_period_height: number,
  max_voting_period_guardians_height: number,
  fund_rotation: number | undefined,
  funding_goal: string | undefined,
  is_localterra: boolean
): Promise<void> {
  // Step 1. Upload all local wasm files and capture the codes for each....
  process.stdout.write("Uploading Registrar Wasm");
  const registrarCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.core}/registrar.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${registrarCodeId}`);

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

  process.stdout.write("Uploading CW3 MultiSig Wasm");
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
      split_to_liquid: undefined,
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

  // Add confirmed TCA Members to the Index Fund SCs approved list
  process.stdout.write("Add confirmed TCA Member to allowed list");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_alliance_member_list: {
        address: tca.key.accAddress,
        member: {
          name: "TCA Member",
          logo: undefined,
          website: "https://angelprotocol.io/app",
        },
        action: "add",
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));

  if (is_localterra) {
    process.stdout.write(
      "Set default vault in Registrar as a placeholder for account creation on localterra & update Index Fund"
    );
    await sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        update_config: {
          default_vault: apTeam.key.accAddress,
          index_fund_contract: indexFund,
        },
      }),
    ]);
    console.log(chalk.green(" Done!"));
  }
}

// Step 4: Create Endowments via the Registrar contract
async function createEndowments(): Promise<void> {
  // endowment #1
  process.stdout.write("Charity Endowment #1 created from the Registrar by the AP Team");
  const charityResult1 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity1.key.accAddress,
        beneficiary: charity1.key.accAddress,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        profile: {
          name: "Test Endowment #1",
          overview: "A wonderful charity endowment that aims to test all the things",
          un_sdg: undefined,
          tier: undefined,
          logo: undefined,
          image: undefined,
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
      },
    }),
  ]);
  endowmentContract1 = charityResult1.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract1}`
  );

  // endowment #2
  process.stdout.write("Charity Endowment #2 created from the Registrar by the AP Team");
  const charityResult2 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity2.key.accAddress,
        beneficiary: charity2.key.accAddress,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        profile: {
          name: "Test Endowment #2",
          overview: "An even better endowment full of butterflies and rainbows",
          un_sdg: undefined,
          tier: undefined,
          logo: undefined,
          image: undefined,
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
      },
    }),
  ]);
  endowmentContract2 = charityResult2.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract2}`
  );

  // endowment #3
  process.stdout.write("Charity Endowment #3 created from the Registrar by the AP Team");
  const charityResult3 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity3.key.accAddress,
        beneficiary: charity3.key.accAddress,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        profile: {
          name: "Test Endowment #3",
          overview: "Shady endowment that will never be approved",
          un_sdg: undefined,
          tier: undefined,
          logo: undefined,
          image: undefined,
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
      },
    }),
  ]);
  endowmentContract3 = charityResult3.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract3}`
  );

  // endowment #4
  process.stdout.write("Charity Endowment #4 created from the Registrar by the AP Team");
  const charityResult4 = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner: charity3.key.accAddress,
        beneficiary: charity3.key.accAddress,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        profile: {
          name: "Vibin' Endowment #4",
          overview: "Global endowment that spreads good vibes",
          un_sdg: undefined,
          tier: undefined,
          logo: undefined,
          image: undefined,
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
      },
    }),
  ]);
  endowmentContract4 = charityResult4.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${endowmentContract4}`
  );
}

async function approveEndowments(): Promise<void> {
  // AP Team approves 3 of 4 newly created endowments
  process.stdout.write("AP Team approves 3 of 4 endowments");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract1,
        status: 1,
        beneficiary: undefined,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract2,
        status: 1,
        beneficiary: undefined,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentContract4,
        status: 1,
        beneficiary: undefined,
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

// Step 5: Index Fund finals setup
async function createIndexFunds(): Promise<void> {
  // Create an initial "Fund" with the two charities created above
  process.stdout.write("Create two Funds with two endowments each");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        name: "Test Fund",
        description: "My first test fund",
        members: [endowmentContract1, endowmentContract2],
        rotating_fund: true,
        split_to_liquid: undefined,
        expiry_time: undefined,
        expiry_height: undefined,
      },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      create_fund: {
        name: "Test Fund #2",
        description: "Another fund to test rotations",
        members: [endowmentContract1, endowmentContract4],
        rotating_fund: true,
        split_to_liquid: undefined,
        expiry_time: undefined,
        expiry_height: undefined,
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

async function createVaults(
  harvest_to_liquid: string,
  tax_per_block: string
): Promise<void> {
  process.stdout.write("Uploading Anchor Vault Wasm");
  const vaultCodeId = await storeCode(terra, apTeam, `${wasm_path.core}/anchor.wasm`);
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${vaultCodeId}`);

  // Anchor Vault - #1
  process.stdout.write("Instantiating Anchor Vault (#1) contract");
  const vaultResult1 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #1",
    symbol: "apANC1",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  anchorVault1 = vaultResult1.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${anchorVault1}`);

  // Anchor Vault - #2 (to better test multistrategy logic)
  process.stdout.write("Instantiating Anchor Vault (#2) contract");
  const vaultResult2 = await instantiateContract(terra, apTeam, apTeam, vaultCodeId, {
    registrar_contract: registrar,
    moneymarket: anchorMoneyMarket ? anchorMoneyMarket : registrar, // placeholder addr for now
    tax_per_block: tax_per_block, // 70% of Anchor's 19.5% earnings collected per block
    name: "AP DP Token - Anchor #2",
    symbol: "apANC2",
    decimals: 6,
    harvest_to_liquid: harvest_to_liquid,
  });
  anchorVault2 = vaultResult2.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
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
    }),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write(
    "Set default vault in Registrar as Anchor Vault #1 && Update Registrar with the Address of the Index Fund contract"
  );
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        default_vault: anchorVault1,
        index_fund_contract: indexFund,
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

// Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract
async function turnOverApTeamMultisig(is_localterra: boolean): Promise<void> {
  process.stdout.write(
    "Turn over Ownership/Admin control of all Core contracts to AP Team MultiSig Contract"
  );
  const msgs = [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_owner: { new_owner: cw3ApTeam },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, indexFund, {
      update_owner: { new_owner: cw3ApTeam },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, endowmentContract1, {
      update_owner: { new_owner: cw3ApTeam },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, endowmentContract2, {
      update_owner: { new_owner: cw3ApTeam },
    }),
    new MsgExecuteContract(apTeam.key.accAddress, endowmentContract3, {
      update_owner: { new_owner: cw3ApTeam },
    }),
  ];
  if (!is_localterra) {
    msgs.push(
      new MsgExecuteContract(apTeam.key.accAddress, anchorVault1, {
        update_owner: { new_owner: cw3ApTeam },
      })
    );
    msgs.push(
      new MsgExecuteContract(apTeam.key.accAddress, anchorVault2, {
        update_owner: { new_owner: cw3ApTeam },
      })
    );
  }
  await sendTransaction(terra, apTeam, msgs);
  console.log(chalk.green(" Done!"));
}
