/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import {
  LcdClient,
  
  Msg,
  MsgExecuteContract,
  Wallet,
} from "@cosmjs/launchpad";
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
  juno: LcdClient,
  apTeam: Wallet,
  registrar: string,
  config: any
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Config");
  await sendTransaction(juno, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      update_config: config,
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
  juno: LcdClient,
  apTeam: Wallet,
  registrar: string,
  msg: any
): Promise<void> {
  process.stdout.write("Create a new endowment via the Registrar");
  const result = await sendTransaction(juno, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, registrar, {
      create_endowment: msg,
    }),
  ]);
  const acct = result.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(chalk.green(` ${acct} - Done!`));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team and trigger harvesting of Accounts for a Vault(s)
//
// SCENARIO:
// AP team needs to send a message to a Vault to trigger a rebalance of Endowment funds,
// moving money from their Locked to Liquid & taking a small tax of DP Tokens as well.
//
//----------------------------------------------------------------------------------------
export async function testCronWalletCanDirectlyHarvestVault(
  juno: LcdClient,
  cron: Wallet,
  vault: string,
  collector_address: string,
  collector_share: string
): Promise<void> {
  process.stdout.write(
    "Test - Cron wallet triggers harvest of single Vault (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(juno, cron, [
      new MsgExecuteContract(cron.key.accAddress, vault, {
        harvest: {
          collector_address,
          collector_share,
        },
      }),
    ])
  );
  console.log(chalk.green(" Passed!"));
}

export async function testAngelTeamCanTriggerVaultsHarvest(
  juno: LcdClient,
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
    sendTransaction(juno, charity1, [
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
    sendTransaction(juno, apTeam, [
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
  juno: LcdClient,
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
  await sendTransaction(juno, apTeam, msgs);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: Can update an Endowment's Entry from the Registrar
//    Possible Values:
//      "address": endowment address
//      "name":    endowment name string | undefined
//      "logo":    endowment logo string | undefined
//      "image":   endowment image string | undefined
//      "owner":   endowment owner address | undefined
//      "tier":    endowment tier number(1, 2, 3) | undefined
//      "un_sdg":  endowment "un_sdg" number (u64) | undefined
//      "endow_type": endowment `EndowmentType` | undefined
//----------------------------------------------------------------------------------------
export async function testUpdateEndowmentsEntry(
  juno: LcdClient,
  apTeam: Wallet,
  registrar: string,
  endowments: any[] // [{ address: "terra...", name: "...", owner: "...", tier: "", un_sdg: "", endow_type: "...", logo: "...", image: "..." }]
): Promise<void> {
  process.stdout.write("AP Team updates endowments type(EndowmentEntry info)");
  let msgs: Msg[] = [];
  endowments.forEach((endow) => {
    msgs.push(
      new MsgExecuteContract(apTeam.key.accAddress, registrar, {
        update_endowment_entry: {
          endowment_addr: endow.address,
          name: endow.name,
          logo: endow.logo,
          image: endow.image,
          owner: endow.owner,
          tier: endow.tier,
          un_sdg: endow.un_sdg,
          endow_type: endow.endow_type,
        },
      })
    );
  });
  await sendTransaction(juno, apTeam, msgs);
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team can trigger migration of all Account SC Endowments from Registrar
//----------------------------------------------------------------------------------------

export async function testMigrateAllAccounts(
  juno: LcdClient,
  apTeam: Wallet,
  registrar: string
): Promise<void> {
  process.stdout.write(
    "Test - AP Team can trigger migration of all Account SC Endowments from Registrar"
  );
  await expect(
    sendTransaction(juno, apTeam, [
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
  juno: LcdClient,
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
  juno: LcdClient,
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
  juno: LcdClient,
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
  juno: LcdClient,
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
  juno: LcdClient,
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
  juno: LcdClient,
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
  juno: LcdClient,
  registrar: string,
  Vault1: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await terra.wasm.contractQuery(registrar, {
    vault: {
      vault_addr: Vault1,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
