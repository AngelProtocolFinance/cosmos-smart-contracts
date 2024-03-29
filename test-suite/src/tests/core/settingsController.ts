/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendTransaction,
  sendMessageViaCw3Endowment,
} from "../../utils/helpers/juno";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

chai.use(chaiAsPromised);
const { expect } = chai;

export async function testUpdateSettingsControllerConfig(
  juno: SigningCosmWasmClient,
  apTeamAddr: string,
  settingsControllerContract: string,
  newConfigMsg: any
): Promise<void> {
  process.stdout.write(
    "Test - ApTeam can update the SettingsController config"
  );
  await sendTransaction(juno, apTeamAddr, settingsControllerContract, {
    update_config: newConfigMsg,
  });
  console.log(chalk.green(" Passed!"));
}

export async function testSetupDao(
  juno: SigningCosmWasmClient,
  charity: string,
  accountsContract: string,
  settingsControllerContract: string,
  endowmentId: number,
  setupDaoMsg: any
): Promise<void> {
  process.stdout.write("Test - Endowment owner can setup the dao");

  const res = await juno.queryContractSmart(accountsContract, {
    endowment: { id: endowmentId },
  });
  const cw3 = res.owner as string;

  await sendMessageViaCw3Endowment(
    juno,
    charity,
    cw3,
    settingsControllerContract,
    {
      setup_dao: {
        endowment_id: endowmentId,
        setup: setupDaoMsg,
      },
    }
  );
  console.log(chalk.green(" Passed!"));
}

export async function testSetupDonationMatch(
  juno: SigningCosmWasmClient,
  charity: string,
  accountsContract: string,
  settingsControllerContract: string,
  endowmentId: number,
  setupMsg: any
): Promise<void> {
  process.stdout.write("Test - Endowment owner can setup the DonationMatch");

  const res = await juno.queryContractSmart(accountsContract, {
    endowment: { id: endowmentId },
  });
  const cw3 = res.owner as string;

  await sendMessageViaCw3Endowment(
    juno,
    charity,
    cw3,
    settingsControllerContract,
    {
      setup_donation_match: {
        endowment_id: endowmentId,
        setup: setupMsg,
      },
    }
  );
  console.log(chalk.green(" Passed!"));
}

export async function testUpdateDelegate(
  juno: SigningCosmWasmClient,
  charity: string,
  accountsContract: string,
  settingsControllerContract: string,
  update_delegate_msg: any
): Promise<void> {
  process.stdout.write("Test - Endowment owner can update the delegate");

  const res = await juno.queryContractSmart(accountsContract, {
    endowment: { id: update_delegate_msg.id },
  });
  const cw3 = res.owner as string;

  await sendMessageViaCw3Endowment(
    juno,
    charity,
    cw3,
    settingsControllerContract,
    {
      update_delegate: {
        endowment_id: update_delegate_msg.id,
        setting: update_delegate_msg.setting,
        action: update_delegate_msg.action,
        delegate_address: update_delegate_msg.delegate_address,
        delegate_expiry: update_delegate_msg.delegate_expiry,
      },
    }
  );
  console.log(chalk.green(" Passed!"));
}

export async function testQuerySettingsControllerConfig(
  juno: SigningCosmWasmClient,
  settingsControllerContract: string
): Promise<void> {
  process.stdout.write("Test - Query SettingsController config\n");
  const config = await juno.queryContractSmart(settingsControllerContract, {
    config: {},
  });

  console.log(config);
  console.log(chalk.green(" Passed!"));
}

export async function testQuerySettingsControllerEndowSettings(
  juno: SigningCosmWasmClient,
  settingsControllerContract: string,
  endowmentId: number
): Promise<void> {
  process.stdout.write(
    `Test - Query SettingsController EndowmentSettings for Endow ID ${endowmentId}`
  );
  const endowmentSettings = await juno.queryContractSmart(
    settingsControllerContract,
    {
      endowment_settings: {
        id: endowmentId,
      },
    }
  );

  console.log(endowmentSettings);
  console.log(chalk.green(" Passed!"));
}

export async function testQuerySettingsControllerEndowPermissions(
  juno: SigningCosmWasmClient,
  settingsControllerContract: string,
  endowmentId: number,
  updater: string
): Promise<void> {
  process.stdout.write(
    `Test - Query SettingsController EndowmentPermissions for Endow ID ${endowmentId}`
  );
  const endowmentSettings = await juno.queryContractSmart(
    settingsControllerContract,
    {
      endowment_permissions: {
        id: endowmentId,
        updater: updater,
      },
    }
  );

  console.log(endowmentSettings);
  console.log(chalk.green(" Passed!"));
}

export async function testQuerySettingsControllerEndowController(
  juno: SigningCosmWasmClient,
  settingsControllerContract: string,
  endowmentId: number
): Promise<void> {
  process.stdout.write(
    `Test - Query SettingsController EndowmentController for Endow ID ${endowmentId}`
  );
  const endowmentSettings = await juno.queryContractSmart(
    settingsControllerContract,
    {
      endowment_controller: {
        id: endowmentId,
      },
    }
  );

  console.log(endowmentSettings);
  console.log(chalk.green(" Passed!"));
}
