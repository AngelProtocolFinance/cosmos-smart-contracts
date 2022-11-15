/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendMessageViaCw3Proposal, sendTransaction } from "../../../utils/juno/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update registrar configs
//
// SCENARIO:
// Cw3ApTeam needs to update registrar config
//
//----------------------------------------------------------------------------------------
export async function testUpdatingRegistrarConfigs(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3ApTeam: string,
  registrar: string,
  config: any
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Config");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3ApTeam, registrar, {
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

// TEST: Update registrar network connections
//
// SCENARIO:
// Cw3ApTeam needs to update registrar network connections
//
//----------------------------------------------------------------------------------------
export async function testUpdatingRegistrarNetworkConnections(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3ApTeam: string,
  registrar: string,
  network_info: any, // NetworkInfo: { name: string, chain_id: string, ibc_channel: string | undefined, ica_address: string | undefined, gas_limit: number | undefined }
  action: string,  // Should be "add" or "remove"
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Network");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3ApTeam, registrar, {
    update_network_connections: {
      network_info,
      action,
    },
  });
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

  const res = await juno.queryContractSmart(registrar, { config: {} });
  const cw3 = await res.owner as string;

  await expect(
    sendMessageViaCw3Proposal(juno, apTeam, cw3, registrar, {
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

  const res = await juno.queryContractSmart(registrar, { config: {} });
  const cw3 = await res.owner as string;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, registrar, {
    update_network_connections: {
      network_info: info.network_info,
      action: info.action,
    }
  },
  );
  console.log(chalk.green(" Passed!"));
}

// TEST: Update registrar owner
//
// SCENARIO:
// Cw3ApTeam needs to update registrar config owner
//
//----------------------------------------------------------------------------------------
export async function testUpdatingRegistrarUpdateOwner(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3ApTeam: string,
  registrar: string,
  new_owner: string,
): Promise<void> {
  process.stdout.write("AP Team updates Registrar Owner");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3ApTeam, registrar, {
    update_owner: {
      new_owner,
    },
  });
  console.log(chalk.green(" Done!"));
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

export async function testQueryRegistrarVaultList(
  juno: SigningCosmWasmClient,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Vault List");
  const result: any = await juno.queryContractSmart(registrar, {
    vault_list: { approved: true },
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

export async function testQueryRegistrarNetworkConnection(
  juno: SigningCosmWasmClient,
  registrar: string,
  chain_id: string
): Promise<void> {
  process.stdout.write("Test - Query Registrar Network connection(s)");
  const result: any = await juno.queryContractSmart(registrar, {
    network_connection: {
      chain_id: chain_id,
    },
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
