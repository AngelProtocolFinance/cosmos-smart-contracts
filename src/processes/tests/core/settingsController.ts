/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
    toEncodedBinary,
    sendTransaction,
    sendTransactionWithFunds,
    sendMessageViaCw3Proposal,
    sendMessagesViaCw3Proposal,
    sendApplicationViaCw3Proposal,
    clientSetup,
    getWalletAddress,
    instantiateContract,
} from "../../../utils/juno/helpers";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

chai.use(chaiAsPromised);
const { expect } = chai;

export async function testQuerySettingsControllerConfig(
    juno: SigningCosmWasmClient,
    settingsControllerContract: string,
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
    endowmentId: number,
): Promise<void> {
    process.stdout.write("Test - Query SettingsController EndowmentSettings for ");
    console.log(endowmentId);
    const endowmentSettings = await juno.queryContractSmart(settingsControllerContract, {
        endowment_settings: {
            id: endowmentId,
        },
    });

    console.log(endowmentSettings);
    console.log(chalk.green(" Passed!"));
}
