/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);

import { fromBase64, fromUtf8 } from "@cosmjs/encoding";
import { assert } from "@cosmjs/utils";
import { AckWithMetadata, RelayInfo, testutils } from "@confio/relayer";

import { Link, Logger } from "@confio/relayer";
import { ChainDefinition, CosmWasmSigner } from "@confio/relayer/build/lib/helpers";

import { localibc, junod, terrad } from "../../config/localIbcConstants";
import { customFundAccount, customSigningClient, customSigningCosmWasmClient, listAccounts, setup } from "../../utils/ibc";
import { SigningCosmWasmClient, toBinary } from "@cosmjs/cosmwasm-stargate";
import { localjuno } from "../../config/localjunoConstants";
import { sendMessageViaCw3Proposal, sendTransactionWithFunds } from "../../utils/juno/helpers";
import { localterra } from "../../config/localterraConstants";
import { LCDClient } from "@terra-money/terra.js";
import { testQueryVaultConfig, testQueryVaultEndowmentBalance, testQueryVaultTokenInfo, testQueryVaultTotalBalance } from "./localterra";


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

    // Restore the existing IBC link.
    const [nodeA, nodeB] = await setup(junod, terrad);
    const link = await Link.createWithExistingConnections(nodeA, nodeB, localibc.conns.juno, localibc.conns.terra);
    // await testIbcQuery(link);

    const junoCharity1Signer = await customSigningCosmWasmClient(junod, localjuno.mnemonicKeys.charity1);

    /* --- EXECUTE tests --- */
    // await testIBCVaultsInvest(
    //     link,
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1, // endowmentId
    //     `locked`, // acct_type
    //     [[localibc.contracts.ibcVaultLocked1, { info: { native: localjuno.denoms.usdc }, amount: "5000" }]],  // Vec<(vault, amount)>
    // );
    // await testIBCVaultsRedeem(
    //     link,
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1, // endowmentId
    //     `locked`, // acct_type
    //     [[localibc.contracts.ibcVaultLocked1, "500000"]],  // Vec<(vault, amount)>
    // );
    // await testIBCVaultReinvestToLocked(
    //     link,
    //     junoCharity1Signer.sign,
    //     junoCharity1Signer.senderAddress,
    //     localjuno.contracts.accounts,
    //     1,
    //     "500000",
    //     localibc.contracts.ibcVaultLiquid1
    // );

    /* ---  QUERY tests --- */
    // let terra = new LCDClient({
    //     URL: localterra.networkInfo.url,
    //     chainID: localterra.networkInfo.chainId,
    // });
    // await testQueryVaultConfig(terra, localterra.contracts.vaultLocked1);
    // await testQueryVaultEndowmentBalance(terra, localterra.contracts.vaultLocked1, 1);
    // await testQueryVaultTokenInfo(terra, localterra.contracts.vaultLocked1);
    // await testQueryVaultTotalBalance(terra, localterra.contracts.vaultLocked1);
}

async function testIBCVaultsInvest(
    link: Link,
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
    const info = await link.relayAll();
    assertPacketsFromA(info, 1, true);
    console.log(info);

    const contractData = parseAcknowledgementSuccess(info.acksFromB[0]);
    console.log(contractData); // check we get { results : ['']} (one message with no data)

    console.log(chalk.green(" Passed!"));
}

async function testIBCVaultsRedeem(
    link: Link,
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
    const info = await link.relayAll();
    assertPacketsFromA(info, 1, true);
    console.log(info);

    const contractData = parseAcknowledgementSuccess(info.acksFromB[0]);
    console.log(contractData); // check we get { results : ['']} (one message with no data)

    console.log(chalk.green(" Passed!"));
}

async function testIBCVaultReinvestToLocked(
    link: Link,
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
    const info = await link.relayAll();
    assertPacketsFromA(info, 1, true);
    console.log(info);

    const contractData = parseAcknowledgementSuccess(info.acksFromB[0]);
    console.log(contractData); // check we get { results : ['']} (one message with no data)

    console.log(chalk.green(" Passed!"));
}

// IBCQuery
async function testIbcQuery(link: Link) {
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

    const info = await link.relayAll();
    assertPacketsFromA(info, 1, true);
    console.log(info);

    const contractData = parseAcknowledgementSuccess(info.acksFromB[0]);
    console.log(contractData); // check we get { results : ['']} (one message with no data)

    const ibcQueryResult = await junoApTeamSigner.sign.queryContractSmart(junoIcaController, {
        latest_query_result: {
            channel_id: channelId,
        }
    });
    console.log(ibcQueryResult);
}



// throws error if not all are success
export function assertAckSuccess(acks: AckWithMetadata[]) {
    for (const ack of acks) {
        const parsed = JSON.parse(fromUtf8(ack.acknowledgement));
        if (parsed.error) {
            throw new Error(`Unexpected error in ack: ${parsed.error}`);
        }
        console.log(parsed);
        if (!parsed.result) {
            throw new Error(`Ack result unexpectedly empty`);
        }
    }
}

// throws error if not all are errors
export function assertAckErrors(acks: AckWithMetadata[]) {
    for (const ack of acks) {
        const parsed = JSON.parse(fromUtf8(ack.acknowledgement));
        if (parsed.result) {
            throw new Error(`Ack result unexpectedly set`);
        }
        if (!parsed.error) {
            throw new Error(`Ack error unexpectedly empty`);
        }
    }
}

export function assertPacketsFromA(relay: RelayInfo, count: number, success: boolean) {
    if (relay.packetsFromA !== count) {
        throw new Error(`Expected ${count} packets, got ${relay.packetsFromA}`);
    }
    if (relay.acksFromB.length !== count) {
        throw new Error(`Expected ${count} acks, got ${relay.acksFromB.length}`);
    }
    if (success) {
        assertAckSuccess(relay.acksFromB);
    } else {
        assertAckErrors(relay.acksFromB);
    }
}

export function assertPacketsFromB(relay: RelayInfo, count: number, success: boolean) {
    if (relay.packetsFromB !== count) {
        throw new Error(`Expected ${count} packets, got ${relay.packetsFromB}`);
    }
    if (relay.acksFromA.length !== count) {
        throw new Error(`Expected ${count} acks, got ${relay.acksFromA.length}`);
    }
    if (success) {
        assertAckSuccess(relay.acksFromA);
    } else {
        assertAckErrors(relay.acksFromA);
    }
}

export function parseAcknowledgementSuccess(ack: AckWithMetadata): unknown {
    const response = JSON.parse(fromUtf8(ack.acknowledgement));
    console.log(response);
    assert(response.result);
    return JSON.parse(fromUtf8(fromBase64(response.result)));
}
