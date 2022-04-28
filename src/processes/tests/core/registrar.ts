/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {
  LCDClient,
  LocalTerra,
  Msg,
  MsgExecuteContract,
  Wallet,
} from "@terra-money/terra.js";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update registrar configs
//
// SCENARIO:
// AP Team Wallet needs to update registrar config
//
//----------------------------------------------------------------------------------------
export async function testUpdatingRegistrarConfigs(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  treasury: string
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Tax Rate");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: {
        treasury,
        tax_rate: "0.2",
      },
    }),
  ]);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Endowment created from the Registrar
//
// SCENARIO:
// User sends request to create a new endowment to the Registrar
//
//----------------------------------------------------------------------------------------
export async function testCreateEndowmentViaRegistrar(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  owner: string
): Promise<void> {
  process.stdout.write("Create a new endowment via the Registrar");
  await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: {
        owner,
        beneficiary: owner,
        withdraw_before_maturity: false,
        maturity_time: undefined,
        maturity_height: undefined,
        guardians_multisig_addr: undefined,
        profile: {
          name: "Test-Suite Endowment",
          overview: "Endowment created from the test-suite integration test",
          un_sdg: 2,
          tier: 3,
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
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team and trigger harvesting of Accounts for a Vault(s)
//
// SCENARIO:
// AP team needs to send a message to a Vault to trigger a rebalance of Endowment funds,
// moving money from their Locked to Liquid & taking a small tax of DP Tokens as well.
//
//----------------------------------------------------------------------------------------
export async function testAngelTeamCanTriggerVaultsHarvest(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  charity1: Wallet,
  registrar: string,
  collector_address: string,
  collector_share: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity1 cannot trigger harvest of all Vaults (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(terra, charity1, [
      new MsgExecuteContract(charity1.key.accAddress, registrar, {
        harvest: {
          collector_address,
          collector_share,
        },
      }),
    ])
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write(
    "Test - AP Team can trigger harvest of all Vaults (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        harvest: {
          collector_address,
          collector_share,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Can update an Endowment's status from the Registrar
//    Possible Status Values:
//    0. Inactive - NO Deposits | NO Withdraws - no beneficiary needed
//    1. Approved - YES Deposits | YES Withdraws - no beneficiary needed
//    2. Frozen - YES Deposits | NO Withdraws - no beneficiary needed
//    3. Closed - NO Deposits | NO Withdraws - IF beneficiary address given: funds go to that wallet
//                ELSE: sent to fund members
//----------------------------------------------------------------------------------------
export async function testUpdateEndowmentsStatus(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string,
  endowments: any[] // [ { address: "terra1....", status: 0|1|2|3, benficiary: "terra1.." | undefined }, ... ]
): Promise<void> {
  process.stdout.write("AP Team updates endowments statuses");
  let msgs: Msg[] = [];
  endowments.forEach((endow) => {
    msgs.push(
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        update_endowment_status: {
          endowment_addr: endow.address,
          status: endow.status,
          beneficiary: endow.beneficiary,
        },
      })
    );
  });
  await sendTransaction(terra, apTeam, msgs);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team can trigger migration of all Account SC Endowments from Registrar
//----------------------------------------------------------------------------------------

export async function testMigrateAllAccounts(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  registrar: string
): Promise<void> {
  process.stdout.write(
    "Test - AP Team can trigger migration of all Account SC Endowments from Registrar"
  );
  await expect(
    sendTransaction(terra, apTeam, [
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        migrate_accounts: {},
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryRegistrarConfig(
  terra: LocalTerra | LCDClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar config and get proper result");
  const result: any = await terra.wasm.contractQuery(registrar, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentDetails(
  terra: LocalTerra | LCDClient,
  registrar: string,
  endowment: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Endowment Details/Status");
  const result: any = await terra.wasm.contractQuery(registrar, {
    endowment: { endowment_addr: endowment },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentList(
  terra: LocalTerra | LCDClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    endowment_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultList(
  terra: LocalTerra | LCDClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedVaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultRateList(
  terra: LocalTerra | LCDClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Approved Vault Exchange Rate List");
  const result: any = await terra.wasm.contractQuery(registrar, {
    approved_vault_rate_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVaultList(
  terra: LocalTerra | LCDClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar VaultList");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVault(
  terra: LocalTerra | LCDClient,
  registrar: string,
  anchorVault1: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault: {
      vault_addr: anchorVault1,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
