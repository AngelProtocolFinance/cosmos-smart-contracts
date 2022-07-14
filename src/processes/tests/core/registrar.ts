/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update registrar configs
//
// SCENARIO:
// AP Team string needs to update registrar config
//
//----------------------------------------------------------------------------------------
export async function testUpdatingRegistrarConfigs(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  config: any
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Config");
  await sendTransaction(juno, apTeam, registrar, {
    update_config: config,
  });
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
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  msg: any
): Promise<void> {
  process.stdout.write("Create a new endowment via the Registrar");
  const result = await sendTransaction(juno, apTeam, registrar, {
    create_endowment: msg,
  });
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
export async function testCronstringCanDirectlyHarvestVault(
  juno: SigningCosmWasmClient,
  cron: string,
  vault: string,
  collector_address: string,
  collector_share: string
): Promise<void> {
  process.stdout.write(
    "Test - Cron wallet triggers harvest of single Vault (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(juno, cron, vault, {
      harvest: {
        collector_address,
        collector_share,
      },
    })
  );
  console.log(chalk.green(" Passed!"));
}

export async function testAngelTeamCanTriggerVaultsHarvest(
  juno: SigningCosmWasmClient,
  apTeam: string,
  charity1: string,
  registrar: string,
  collector_address: string,
  collector_share: string
): Promise<void> {
  process.stdout.write(
    "Test - Charity1 cannot trigger harvest of all Vaults (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(juno, charity1, registrar, {
      harvest: {
        collector_address,
        collector_share,
      },
    })
  ).to.be.rejectedWith("Request failed with status code 400");
  console.log(chalk.green(" Failed!"));

  process.stdout.write(
    "Test - AP Team can trigger harvest of all Vaults (Locked to Liquid Account)"
  );
  await expect(
    sendTransaction(juno, apTeam, registrar, {
      harvest: {
        collector_address,
        collector_share,
      },
    })
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
export async function testUpdateEndowmentStatus(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  endowmentStatus: any, // [ { address: "juno1....", status: 0|1|2|3, benficiary: "juno1.." | undefined }, ... ]
): Promise<void> {
  process.stdout.write("AP Team updates endowment's status");
  expect(
    await sendTransaction(juno, apTeam, registrar, {
      update_endowment_status: {
        endowment_addr: endowmentStatus.address,
        status: endowmentStatus.status,
        beneficiary: endowmentStatus.beneficiary,
      },
    })
  );
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
export async function testUpdateEndowmentEntry(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  endowmentEntry: any, // [{ address: "juno...", name: "...", owner: "...", tier: "", un_sdg: "", endow_type: "...", logo: "...", image: "..." }]
): Promise<void> {
  process.stdout.write("AP Team updates endowment's EndowmentEntry info");
  expect(
    await sendTransaction(juno, apTeam, registrar, {
      update_endowment_entry: {
        endowment_addr: endowmentEntry.address,
        name: endowmentEntry.name,
        logo: endowmentEntry.logo,
        image: endowmentEntry.image,
        owner: endowmentEntry.owner,
        tier: endowmentEntry.tier,
        un_sdg: endowmentEntry.un_sdg,
        endow_type: endowmentEntry.endow_type,
      },
    })
  );
  console.log(chalk.green(" Done!"));
}

//----------------------------------------------------------------------------------------
// TEST: AP Team can trigger migration of all Account SC Endowments from Registrar
//----------------------------------------------------------------------------------------

export async function testMigrateAllAccounts(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string
): Promise<void> {
  process.stdout.write(
    "Test - AP Team can trigger migration of all Account SC Endowments from Registrar"
  );
  await expect(
    sendTransaction(juno, apTeam, registrar, { migrate_accounts: {} })
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Only owner can update owner/admin of the Registrar Config.
//
// SCENARIO:
// (config)Owner updates the owner address in registrar config.
//
//----------------------------------------------------------------------------------------
export async function testRegistrarUpdateOwner(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  new_owner: string,
): Promise<void> {
  process.stdout.write(
    "Test - Owner can set new_owner address in Registrar"
  );

  await expect(
    sendTransaction(juno, apTeam, registrar, {
        update_owner: { new_owner },
      },
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Only owner can update the "EndowTypeFees"
//
// SCENARIO:
// (config)Owner updates both "EndowTypeFees"
//
//----------------------------------------------------------------------------------------
export async function testUpdateEndowTypeFees(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  fees: any, // { endowtype_charity: string | undefined, endowtype_normal: string | undefined }
): Promise<void> {
  process.stdout.write(
    "Test - Owner can update EndowTypeFees in Registrar"
  );

  await expect(
    sendTransaction(juno, apTeam, registrar, {
        update_endow_type_fees: {
          endowtype_charity: fees.endowtype_charity,
          endowtype_normal: fees.endowtype_normal,
        }
      },
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Only owner can update the "NetworkConnection(s)"
//
// SCENARIO:
// (config)Owner updates both "EndowTypeFees"
//
//----------------------------------------------------------------------------------------
export async function testUpdateNetworkConnections(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  info: any, // { network_info: obj, action: "add" | "remove" }
): Promise<void> {
  process.stdout.write(
    "Test - Owner can update network_connections in Registrar"
  );

  await expect(
    sendTransaction(juno, apTeam, registrar, {
        update_network_connections: {
          network_info: info.network_info,
          action: info.action,
        }
      },
    )
  ).to.be.ok;
  console.log(chalk.green(" Passed!"));
}


//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryRegistrarConfig(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar config and get proper result");
  const result: any = await juno.queryContractSmart(registrar, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentDetails(
  juno: SigningCosmWasmClient,
  registrar: string,
  endowment: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Endowment Details/Status");
  const result: any = await juno.queryContractSmart(registrar, {
    endowment: { endowment_addr: endowment },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarEndowmentList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar EndowmentList");
  const result: any = await juno.queryContractSmart(registrar, {
    endowment_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar ApprovedVaultList");
  const result: any = await juno.queryContractSmart(registrar, {
    approved_vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarApprovedVaultRateList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Approved Vault Exchange Rate List");
  const result: any = await juno.queryContractSmart(registrar, {
    approved_vault_rate_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVaultList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar VaultList");
  const result: any = await juno.queryContractSmart(registrar, {
    vault_list: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryRegistrarVault(
  juno: SigningCosmWasmClient,
  registrar: string,
  Vault1: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault");
  const result: any = await juno.queryContractSmart(registrar, {
    vault: {
      vault_addr: Vault1,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
