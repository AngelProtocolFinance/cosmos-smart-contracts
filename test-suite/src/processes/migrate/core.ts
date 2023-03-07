/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendMessageViaCw3Proposal,
  sendMessagesViaCw3Proposal,
  sendTransaction,
  storeCode,
  migrateContract,
  storeAndMigrateContract,
  storeAndInstantiateContract,
  toEncodedBinary,
} from "../../utils/juno/helpers";
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
  cw4GrpReviewTeam: string,
  cw3ReviewTeam: string,
  swapRouter: string,
  settingsController: string,
  donationMatching: string,
  giftcards: string,
  vaultContracts: string[],
  axelarGateway: string,
  axelarIbcChannel: string
): Promise<void> {
  // await storeAndMigrateContract(juno, apTeam, registrar, "registrar.wasm", {
  //     axelar_gateway: axelarGateway,
  //     axelar_ibc_channel: axelarIbcChannel,
  //     accounts_settings_controller: settingsController,
  // });
  await storeAndMigrateContract(juno, apTeam, accounts, "accounts.wasm");
  // await storeAndMigrateContract(juno, apTeam, settingsController, 'settings_controller.wasm');
  // await storeAndMigrateContract(juno, apTeam, indexFund, 'index_fund.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw4GrpApTeam, 'cw4_group.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ApTeam, 'cw3_apteam.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw4GrpReviewTeam, 'cw4_group.wasm');
  // await storeAndMigrateContract(juno, apTeam, cw3ReviewTeam, 'cw3_applications.wasm');
  // await storeAndMigrateContract(juno, apTeam, giftcards, 'fundraising.wasm');
  // await storeAndMigrateContract(juno, apTeam, giftcards, 'gift_cards.wasm');
  // await storeAndMigrateContract(juno, apTeam, swapRouter, 'swap_router.wasm');
  // await migrateVaults(juno, apTeam, vaultContracts);
  await migrateEndowmentCw3s(
    juno,
    apTeam,
    cw3ApTeam,
    registrar,
    accounts,
    1,
    25
  );
  // await migrateEndowmentCw3s(juno, apTeam, cw3ApTeam, registrar, accounts, 1, 50);
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
    path.resolve(__dirname, `${wasm_path.mock_vault}/mock_vault.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write("Migrate Vault contracts\n");
  let prom = Promise.resolve();
  let id = 1;
  vaults.forEach((vault) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(async () => {
      await migrateContract(juno, apTeam, vault, codeId, {});
      console.log(chalk.green(`Vault ${id++} of ${vaults.length} - Done!`));
    });
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
  start_id: number,
  max_process: number
): Promise<void> {
  process.stdout.write("Uploading Endowment CW3 Wasm");
  const codeId = await storeCode(
    juno,
    apTeam,
    path.resolve(__dirname, `${wasm_path.core}/cw3_endowment.wasm`)
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${codeId}`);

  process.stdout.write(
    "Ensure Registrar has the latest Endowment CW3 Wasm code set"
  );
  await sendMessageViaCw3Proposal(juno, apTeam, cw3ApTeam, registrar, {
    update_config: { cw3_code: codeId },
  });

  process.stdout.write(
    `Migrate all Endowment CW3 contracts (start ID: ${start_id}; No. to process: ${max_process})\n`
  );
  let prom = Promise.resolve();
  const final_msgs: any[] = [];
  const ids_range = new Array(max_process).fill(0).map((d, i) => i + start_id);
  ids_range.forEach((id) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(async () => {
      console.log(
        chalk.yellow(`Building migration message for Endowment ID: ${id}`)
      );
      const res = await juno.queryContractSmart(accounts, {
        endowment: { id },
      });
      const cw3 = res.owner as string;
      // push a new migration message to the array
      final_msgs.push({
        wasm: {
          migrate: {
            contract_addr: cw3,
            new_code_id: codeId,
            msg: toEncodedBinary({}),
          },
        },
      });
    });
  });
  await prom;
  await sendMessagesViaCw3Proposal(
    juno,
    apTeam,
    cw3ApTeam,
    "Migrate several Endowment multisig contracts",
    final_msgs
  );
  console.log(chalk.green(" Done!"));
}
