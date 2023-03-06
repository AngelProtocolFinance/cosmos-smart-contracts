/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { sendTransaction } from "../../../utils/juno/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Update community config
//
// SCENARIO:
// Pleb cannot update contract config, only gov contract can update config
//
//----------------------------------------------------------------------------------------
export async function testCommunityUpdateConfig(
    juno: SigningCosmWasmClient,
    apTeam: string,
    pleb: string,
    govContract: string,
    communityContract: string,
    spend_limit: string | undefined,
    new_gov_contract: string | undefined
): Promise<void> {
    process.stdout.write("Test - Pleb cannot update community config");

    await expect(
        sendTransaction(juno, pleb, communityContract, {
            update_config: { spend_limit, gov_contract: new_gov_contract },
        })
    ).to.be.rejectedWith("Request failed with status code 400");
    console.log(chalk.green(" Failed!"));

    process.stdout.write("Test - Only gov contract can update community config");

    await expect(
        sendTransaction(juno, govContract, communityContract, {
            update_config: { spend_limit, gov_contract: new_gov_contract },
        })
    );
    console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// TEST: Spend
//
// SCENARIO:
// Owner can execute spend operation to send
// `amount` of HALO token to `recipient` for community purpose
//
//----------------------------------------------------------------------------------------
export async function testCommunitySpend(
    juno: SigningCosmWasmClient,
    apTeam: string,
    govContract: string,
    communityContract: string,
    receipient: string,
    amount: string
): Promise<void> {
    process.stdout.write("Test - Send `amount` of HALO token to `receipient` for community purpose");

    await expect(
        sendTransaction(juno, govContract, communityContract, {
            spend: { receipient, amount },
        })
    );
    console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------
export async function testQueryCommunityConfig(juno: SigningCosmWasmClient, communityContract: string): Promise<void> {
    process.stdout.write("Test - Query Community Config");
    const result: any = await juno.queryContractSmart(communityContract, {
        config: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}
