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
// TEST: Only owner can update the "Fees"
//
// SCENARIO:
// (config)Owner updates both "Fees"
//
//----------------------------------------------------------------------------------------
export async function testUpdateFees(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  info: any,
): Promise<void> {
  process.stdout.write(
    "Test - Owner can update EndowTypeFees in Registrar"
  );

  const config = await juno.queryContractSmart(registrar, { config: {} });
  const cw3 = await config.owner as string;

  await sendMessageViaCw3Proposal(juno, apTeam, cw3, registrar, {
    update_fees: {
      fees: info.fees,
    }
  });
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
