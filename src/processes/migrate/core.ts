/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendMessageViaCw3Proposal, sendMessagesViaCw3Proposal, sendTransaction, storeCode, migrateContract, storeAndMigrateContract, toEncodedBinary } from "../../utils/juno/helpers";
import { wasm_path } from "../../config/wasmPaths";

// -----------------------------
// Base functions to migrate contracts with
// -----------------------------
export async function migrateCore(
  juno: SigningCosmWasmClient,
  apTeam: string,
  registrar: string,
  indexFund: string,
  accounts: string,
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  cw3ReviewTeam: string,
  swapRouter: string,
  settingsController: string,
  vaultContracts: string[],
): Promise<void> {
  // run the migrations desired
  // await migrateEndowmentCw3s(juno, apTeam, cw3ApTeam, registrar, accounts, 0);
  // await migrateVaults(juno, apTeam, vaultContracts);
  // await storeAndMigrateContract(juno, apTeam, registrar, 'registrar.wasm', { endowtype_fees: { endowtype_charity: undefined, endowtype_normal: undefined }, collector_addr: undefined });
  // await storeAndMigrateContract(juno, apTeam, cw4GrpApTeam, 'cw4_group.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ApTeam, 'cw3_apteam.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ReviewTeam, 'cw3_applications.wasm');
  // await storeAndMigrateContract(juno, apTeam, indexFund, 'index_fund.wasm');
  // await storeAndMigrateContract(juno, apTeam, accounts, 'accounts.wasm', { settings_controller_contract: undefined });
  // await storeAndMigrateContract(juno, apTeam, swapRouter, 'swap_router.wasm');
  // await storeAndMigrateContract(juno, apTeam, settingsController, 'settings_controller.wasm');
}

// -------------------------------------------------
//  Migrate vaults
//--------------------------------------------------
async function migrateVaults(
  juno: SigningCosmWasmClient,
  apTeam: string,
  vaults: string[]
): Promise<void> {
  process.stdout.write("Uploading Vault Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/loopswap_vault.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  vaults.forEach((vault) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            await migrateContract(juno, apTeam, vault, codeId, {});
            console.log(chalk.green(`Vault ${id++} of ${vaults.length} - Done!`));
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });

  await prom;
  console.log(chalk.green(" Done!"));
}

// -------------------------------------------------
//  Migrate Endowment CW3s
//--------------------------------------------------
async function migrateEndowmentCw3s(
  juno: SigningCosmWasmClient,
  apTeam: string,
  cw3ApTeam: string,
  registrar: string,
  accounts: string,
  max_id: number,
): Promise<void> {
  process.stdout.write("Uploading Endowment CW3 Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/cw3_endowment.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Ensure Registrar has the latest Endowment CW3 Wasm code set");
  await sendMessageViaCw3Proposal(juno, apTeam, cw3ApTeam, registrar, { update_config: { cw3_code: codeId } });

  process.stdout.write("Migrate all Endowment CW3 contracts\n");
  let prom = Promise.resolve();
  let final_msgs: any[] = [];
  const ids_range = new Array(max_id).fill(0).map((d, i) => i + 1);
  ids_range.forEach((id) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            console.log(chalk.yellow(`Building migration message for Endowment ID: ${id}`));
            const res = await juno.queryContractSmart(accounts, { endowment: { id } });
            const cw3 = res.owner as string;
            // push a new migration message to the array
            final_msgs.push({
              wasm: {
                migrate: {
                  contract_addr: cw3,
                  new_code_id: codeId,
                  msg: toEncodedBinary({})
                },
              },
            });
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });
  await prom;
  await sendMessagesViaCw3Proposal(juno, apTeam, cw3ApTeam, "Migrate several Endowment multisig contracts", final_msgs);
  console.log(chalk.green(" Done!"));
}
