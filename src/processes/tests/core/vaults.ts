/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendMessageViaCw3Proposal, sendTransaction } from "../../../utils/helpers";

chai.use(chaiAsPromised);

//----------------------------------------------------------------------------------------
// Execution tests
//----------------------------------------------------------------------------------------
export async function testVaultHarvest(
  juno: SigningCosmWasmClient,
  sender: string,
  vault: string,
): Promise<void> {
  process.stdout.write("Test - Keeper harvests the vault")
  await sendTransaction(juno, sender, vault, { harvest: {}});
  console.log(chalk.green(" Passed!"));
}

export async function testVaultReinvestToLocked(
  juno: SigningCosmWasmClient,
  sender: string,
  accountsContract: string,
  endowmentId: number,
  amount: string,
  vault_addr: string,
): Promise<void> {
  process.stdout.write("Test - Liquid vault reinvests the LP to locked vault");

  const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId }});
  const cw3 = res.owner as string;

  await sendMessageViaCw3Proposal(juno, sender, cw3, accountsContract, 
    { 
      reinvest_to_locked: { 
        id: endowmentId, 
        amount: amount, 
        vault_addr: vault_addr,
      }
    }
  );
  console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
  juno: SigningCosmWasmClient,
  vault: string
): Promise<void> {
  process.stdout.write("Test - Query Vault Config");
  const result: any = await juno.queryContractSmart(vault, {
    vault_config: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
