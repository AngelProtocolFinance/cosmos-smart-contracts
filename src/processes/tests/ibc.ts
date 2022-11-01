/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);

// IBC-related imports

import { Link, Logger } from "@confio/relayer";
import { ChainDefinition, CosmWasmSigner } from "@confio/relayer/build/lib/helpers";

import { localibc, junod, terrad } from "../../config/localIbcConstants";
import { customFundAccount, customSigningClient, customSigningCosmWasmClient, listAccounts, setup } from "../../utils/ibc";
import { SigningCosmWasmClient, toBinary } from "@cosmjs/cosmwasm-stargate";
import { localjuno } from "../../config/localjunoConstants";
import { sendMessageViaCw3Proposal, sendTransactionWithFunds } from "../../utils/juno/helpers";
import { localterra } from "../../config/localterraConstants";
import { LCDClient } from "@terra-money/terra.js";


// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let junoIcaController: string;
let junoIcaHost: string;

let terraIcaController1: string;
let terraIcaController2: string;
let terraIcaHost: string;

let junoApTeamSigner: CosmWasmSigner;

export async function testExecuteIBC(
    ibc: {
        junoIcaController: string,
        junoIcaHost: string,
        terraIcaController1: string,
        terraIcaController2: string,
        terraIcaHost: string,
    }
): Promise<void> {
    console.log(chalk.yellow("\nStep 2. Running Tests"));

    junoIcaController = ibc.junoIcaController;
    junoIcaHost = ibc.junoIcaHost;

    terraIcaController1 = ibc.terraIcaController1;
    terraIcaController2 = ibc.terraIcaController2;
    terraIcaHost = ibc.terraIcaHost;

    junoApTeamSigner = await customSigningCosmWasmClient(junod, localjuno.mnemonicKeys.apTeam);

    // // Restore the existing IBC link.
    // const [nodeA, nodeB] = await setup(junod, terrad);
    // const link = await Link.createWithExistingConnections(nodeA, nodeB, localibc.conns.juno, localibc.conns.terra);
    // await testIbcQuery(link);

    const junoCharity1Signer = await customSigningCosmWasmClient(junod, localjuno.mnemonicKeys.charity1);

    /* --- EXECUTE tests --- */
    // await testIBCVaultsInvest(
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1, // endowmentId
    //     `locked`, // acct_type
    //     [[localibc.contracts.ibcVaultLocked1, { info: { native: localjuno.denoms.usdc }, amount: "5000" }]],  // Vec<(vault, amount)>
    // );
    // await testIBCVaultsRedeem(
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1, // endowmentId
    //     `locked`, // acct_type
    //     [[localibc.contracts.ibcVaultLocked1, "500000"]],  // Vec<(vault, amount)>
    // );
    // await testIBCVaultReinvestToLocked(
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1,
    //     "500000",
    //     localibc.contracts.ibcVaultLiquid1
    // );

    // await testVaultHarvest(terra, apTeam, vaultLocked1);


    /* ---  QUERY tests --- */
    // await testQueryVaultConfig(terra, vaultLocked1);
    // await testQueryVaultEndowmentBalance(terra, vaultLocked1, 1);
    // await testQueryVaultTokenInfo(terra, vaultLocked1);
    // await testQueryVaultTotalBalance(terra, vaultLocked1);
}

async function testIBCVaultsInvest(
    juno: SigningCosmWasmClient,
    accountsOwner: string,
    accountsContract: string,
    endowmentId: number,
    acct_type: string,
    vaults: any,
): Promise<void> {
    process.stdout.write("IBC Test - Send amount to a single Endowment Account");

    const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
    const cw3 = res.owner as string;

    await sendMessageViaCw3Proposal(juno, accountsOwner, cw3, accountsContract, {
        vaults_invest: {
            id: endowmentId,
            acct_type: acct_type,
            vaults,
        },
    });
    console.log(chalk.green(" Passed!"));
}

async function testIBCVaultsRedeem(
    juno: SigningCosmWasmClient,
    accountsOwner: string,
    accountsContract: string,
    endowmentId: number,
    acct_type: string,
    vaults: any,
): Promise<void> {
    process.stdout.write("IBC Test -Redeem endowment");

    const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
    const cw3 = res.owner as string;

    await sendMessageViaCw3Proposal(juno, accountsOwner, cw3, accountsContract, {
        vaults_redeem: {
            id: endowmentId,
            acct_type: acct_type,
            vaults,
        },
    });
    console.log(chalk.green(" Passed!"));
}

async function testIBCVaultReinvestToLocked(
    juno: SigningCosmWasmClient,
    sender: string,
    accountsContract: string,
    endowmentId: number,
    amount: string,
    vault_addr: string,
): Promise<void> {
    process.stdout.write("Test - Liquid vault reinvests the LP to locked vault");

    const res = await juno.queryContractSmart(accountsContract, { endowment: { id: endowmentId } });
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

// IBCQuery
async function testIbcQuery(link: Link) {
    await link.relayAll();

    const accounts = await listAccounts(junoApTeamSigner, junoIcaController);
    // console.log("accounts query: ", accounts);
    const { remote_addr: remoteAddr, channel_id: channelId } = accounts[0];

    await junoApTeamSigner.sign.execute(
        junoApTeamSigner.senderAddress,
        junoIcaController,
        {
            ibc_query: {
                channel_id: channelId,
                msgs: [{ smart: { msg: toBinary({ list_accounts: {} }), contract_addr: terraIcaHost } }]
            }
        },
        "auto"
    );

    const ibcQueryResult = await junoApTeamSigner.sign.queryContractSmart(junoIcaController, {
        latest_query_result: {
            channel_id: channelId,
        }
    });
    console.log(ibcQueryResult);
}
