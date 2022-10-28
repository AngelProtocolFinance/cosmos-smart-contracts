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


// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let junoIcaController: string;
let junoIcaControllerPort: string;
let junoIcaHost: string;
let junoIcaHostPort: string;

let terraIcaController: string;
let terraIcaControllerPort: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

let junoIbcSigner: CosmWasmSigner;

export async function testExecuteIBC(
    ibc: {
        junoIcaController: string,
        junoIcaControllerPort: string,
        junoIcaHost: string,
        junoIcaHostPort: string,
        terraIcaController: string,
        terraIcaControllerPort: string,
        terraIcaHost: string,
        terraIcaHostPort: string,
    }
): Promise<void> {
    console.log(chalk.yellow("\nStep 2. Running Tests"));

    junoIcaController = ibc.junoIcaController;
    junoIcaControllerPort = ibc.junoIcaControllerPort;
    junoIcaHost = ibc.junoIcaHost;
    junoIcaHostPort = ibc.junoIcaHostPort;

    terraIcaController = ibc.terraIcaController;
    terraIcaControllerPort = ibc.terraIcaControllerPort;
    terraIcaHost = ibc.terraIcaHost;
    terraIcaHostPort = ibc.terraIcaHostPort;

    junoIbcSigner = await customSigningCosmWasmClient(junod, localibc.mnemonicKeys.junoIbcClient);

    // Restore the existing IBC link.
    const [nodeA, nodeB] = await setup(junod, terrad);

    const link = await Link.createWithExistingConnections(nodeA, nodeB, localibc.conns.juno, localibc.conns.terra);
    const ics20 = {
        junoTerra: {
            juno: localibc.ics20.junoTerra.juno,
            terra: localibc.ics20.junoTerra.terra,
        },
        terraJuno: {
            terra: localibc.ics20.terraJuno.terra,
            juno: localibc.ics20.terraJuno.juno,
        }
    };
    console.log("ics20: ", ics20);

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

    const accounts = await listAccounts(junoIbcSigner, junoIcaController);
    // console.log("accounts query: ", accounts);
    const { remote_addr: remoteAddr, channel_id: channelId } = accounts[0];

    const ibcQuery = await junoIbcSigner.sign.execute(
        junoIbcSigner.senderAddress,
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
