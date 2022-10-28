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
import { toBinary } from "@cosmjs/cosmwasm-stargate";
import { localjuno } from "../../config/localjunoConstants";


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

    /* --- EXECUTE tests --- */
    // await testVaultDeposit(terra, apTeam, vaultLocked1, 1, { uluna: 2000000 });
    // await testVaultRedeem(terra, apTeam, vaultLocked1, 1, "500000");
    // await testVaultHarvest(terra, apTeam, vaultLocked1);
    // await testVaultReinvestToLocked(terra, apTeam, vaultLiquid1, 1, "500000");

    /* ---  QUERY tests --- */
    // await testQueryVaultConfig(terra, vaultLocked1);
    // await testQueryVaultEndowmentBalance(terra, vaultLocked1, 1);
    // await testQueryVaultTokenInfo(terra, vaultLocked1);
    // await testQueryVaultTotalBalance(terra, vaultLocked1);

    await testIbcQuery(link);
}

// IBCQuery
async function testIbcQuery(link: Link) {
    await link.relayAll();

    const accounts = await listAccounts(junoApTeamSigner, junoIcaController);
    // console.log("accounts query: ", accounts);
    const { remote_addr: remoteAddr, channel_id: channelId } = accounts[0];

    const ibcQuery = await junoApTeamSigner.sign.execute(
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
    console.log("IbcQuery content: ", ibcQuery);
}
