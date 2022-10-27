/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";

import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
chai.use(chaiAsPromised);

// IBC-related imports
import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";
import { Link, Logger } from "@confio/relayer";
import { ChainDefinition } from "@confio/relayer/build/lib/helpers";

import { localibc, junod, terrad } from "../../config/localIbcConstants";
import { customFundAccount, customSigningClient, customSigningCosmWasmClient, listAccounts } from "../../utils/ibc";
import { toBinary } from "@cosmjs/cosmwasm-stargate";

const IbcVersion = "ica-vaults-v1";
const Ics20Version = "ics20-1";

let junoIcaController: string;
let junoIcaControllerPort: string;
let junoIcaHost: string;
let junoIcaHostPort: string;

let terraIcaController: string;
let terraIcaControllerPort: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

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

    const { link, ics20 } = await customConnSetup(junod, terrad);

    // IBCQuery 
    const junoSigner = await customSigningCosmWasmClient(junod, localibc.mnemonicKeys.junoIbcClient);

    await link.relayAll();
    const accounts = await listAccounts(junoSigner, junoIcaController);
    console.log("accounts query: ", accounts);
    const { remote_addr: remoteAddr, channel_id: channelId } = accounts[0];
    const ibcQuery = await junoSigner.sign.execute(
        junoSigner.senderAddress,
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
}


/**
 * Clone of original "@confio/relayer/testutils/setup" util.  
 * create a connection and channel for simple-ica
 * @param srcConfig Source chain definition
 * @param destConfig Destination chain definition
 * @param logger 
 * @returns Promise<{link, ics20}>
 */
async function customConnSetup(srcConfig: ChainDefinition, destConfig: ChainDefinition, logger?: Logger) {
    // create apps and fund an account
    const mnemonic = localibc.mnemonicKeys.signingClient;

    const src = await customSigningClient(srcConfig, mnemonic);
    const dest = await customSigningClient(destConfig, mnemonic);

    await customFundAccount(destConfig, dest.senderAddress, '4000000');
    await customFundAccount(srcConfig, src.senderAddress, '4000000');

    const link = await Link.createWithNewConnections(src, dest);
    await link.createChannel("A", junoIcaControllerPort, terraIcaHostPort, Order.ORDER_UNORDERED, IbcVersion);
    await link.createChannel("B", terraIcaControllerPort, junoIcaHostPort, Order.ORDER_UNORDERED, IbcVersion);

    // also create a ics20 channel on this connection
    const ics20Info1 = await link.createChannel("A", junod.ics20Port, terrad.ics20Port, Order.ORDER_UNORDERED, Ics20Version);
    const ics20Info2 = await link.createChannel("B", terrad.ics20Port, junod.ics20Port, Order.ORDER_UNORDERED, Ics20Version);

    const ics20 = {
        junoTerra: {
            juno: ics20Info1.src.channelId,
            terra: ics20Info1.dest.channelId,
        },
        terraJuno: {
            terra: ics20Info2.src.channelId,
            juno: ics20Info2.dest.channelId,
        }
    };
    return { link, ics20 };
}
