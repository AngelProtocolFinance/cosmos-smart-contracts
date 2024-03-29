/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendMessageViaCw3Proposal,
  sendTransaction,
} from "../../utils/helpers/juno";

chai.use(chaiAsPromised);

//----------------------------------------------------------------------------------------
// Execution tests
//----------------------------------------------------------------------------------------
export async function testVaultHarvest(
  juno: SigningCosmWasmClient,
  sender: string,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Keeper harvests the vault");
  await sendTransaction(juno, sender, vault, { harvest: {} });
  console.log(chalk.green(" Passed!"));
}

export async function testVaultReinvestToLocked(
  juno: SigningCosmWasmClient,
  sender: string,
  accountsContract: string,
  endowmentId: number,
  amount: string,
  vault_addr: string
): Promise<void> {
  process.stdout.write("Test - Liquid vault reinvests the LP to locked vault");

  const res = await juno.queryContractSmart(accountsContract, {
    endowment: { id: endowmentId },
  });
  const cw3 = res.owner as string;

  await sendMessageViaCw3Proposal(juno, sender, cw3, accountsContract, {
    reinvest_to_locked: {
      id: endowmentId,
      amount: amount,
      vault_addr: vault_addr,
    },
  });
  console.log(chalk.green(" Passed!"));
}

export async function testVaultUpdateConfig(
  juno: SigningCosmWasmClient,
  sender: string,
  vault_addr: string,
  new_config: any | undefined
): Promise<void> {
  process.stdout.write("Test - Vault owner updates the vault config");
  await sendTransaction(juno, sender, vault_addr, {
    update_config: new_config,
  });
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
  juno: SigningCosmWasmClient,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Config\n");
  const result: any = await juno.queryContractSmart(vault, {
    config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultEndowmentBalance(
  juno: SigningCosmWasmClient,
  vault: string,
  endowmentId: number
): Promise<void> {
  process.stdout.write("Test - Query Vault Endowment Balance\n");
  const result: any = await juno.queryContractSmart(vault, {
    balance: { endowment_id: endowmentId },
  });

  console.log(`Endow ID #${endowmentId} balance: ${result}`);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTotalBalance(
  juno: SigningCosmWasmClient,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Total Balance\n");
  const result: any = await juno.queryContractSmart(vault, {
    total_balance: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTokenInfo(
  juno: SigningCosmWasmClient,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Token Info\n");
  const result: any = await juno.queryContractSmart(vault, {
    token_info: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
