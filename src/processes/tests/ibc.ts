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

const IbcVersion = "ica-vaults-v1"; // "simple-ica-v2";

export async function testExecuteIBC(
    terra: LocalTerra | LCDClient, // environment config object 
    apTeam: Wallet,
    apTeam2: Wallet,
    apTeam3: Wallet,
    apTreasury: Wallet,
    vaultLocked1: string,
    vaultLiquid1: string,
    vaultLocked2: string,
    vaultLiquid2: string,

    astroportFactory: string,
    astroportGenerator: string,
    astroportRouter: string,
    astroTokenContract: string,
    astroTokenInitialSupply: string,
    usdcUsdtPair: string,
    usdcUsdtPairLpToken: string,
    usdcUsdtPairUsdcLiquidity: string,
    usdcUsdtPairUsdtLiquidity: string,
): Promise<void> {
    console.log(chalk.yellow("\nStep 2. Running Tests"));

    const { link, ics20 } = await customConnSetup(junod, terrad);

    // IBCQuery 
    const junoSigner = await customSigningCosmWasmClient(junod, localibc.mnemonicKeys.junoIbcClient);

    await link.relayAll();
    const accounts = await listAccounts(junoSigner, junoIcaController);
    console.log("accounts query: ", accounts);
    const { remote_addr: remoteAddr, channel_id: channelId } = accounts[0];
    const ibcQuery = await juno.execute(
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

    // also create a ics20 channel on this connection
    const ics20Info = await link.createChannel("A", junod.ics20Port, terrad.ics20Port, Order.ORDER_UNORDERED, "ics20-1");
    const ics20 = {
        juno: ics20Info.src.channelId,
        terra: ics20Info.dest.channelId,
    };
    return { link, ics20 };
}

//----------------------------------------------------------------------------------------
// Execution tests
//----------------------------------------------------------------------------------------
export async function testVaultDeposit(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowment_id: number,
    coins: any,
): Promise<void> {
    process.stdout.write("Test - Ibc_host deposits to the vault");
    // TODO
    console.log(chalk.green(" Passed!"));
}

export async function testVaultRedeem(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowment_id: number,
    amount: string,
): Promise<void> {
    process.stdout.write("Test - Ibc_host redeems from the vault");
    // TODO
    console.log(chalk.green(" Passed!"));
}

export async function testVaultHarvest(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
): Promise<void> {
    process.stdout.write("Test - Keeper harvests the vault");
    // TODO
    console.log(chalk.green(" Passed!"));
}

export async function testVaultReinvestToLocked(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault: string,
    endowmentId: number,
    amount: string,
): Promise<void> {
    process.stdout.write("Test - Liquid vault reinvests the LP to locked vault");
    // TODO
    console.log(chalk.green(" Passed!"));
}

export async function testVaultUpdateConfig(
    terra: LocalTerra | LCDClient,
    sender: Wallet,
    vault_addr: string,
    new_config: any | undefined,
): Promise<void> {
    process.stdout.write("Test - Vault owner updates the vault config")
    // TODO
    console.log(chalk.green(" Passed!"));
}

//----------------------------------------------------------------------------------------
// Querying tests
//----------------------------------------------------------------------------------------

export async function testQueryVaultConfig(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Config\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        config: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultEndowmentBalance(
    terra: LocalTerra | LCDClient,
    vault: string,
    endowmentId: number,
): Promise<void> {
    process.stdout.write("Test - Query Vault Endowment Balance\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        balance: { endowment_id: endowmentId },
    });

    console.log(`Endow ID #${endowmentId} balance: ${result}`);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTotalBalance(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Total Balance\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        total_balance: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}

export async function testQueryVaultTokenInfo(
    terra: LocalTerra | LCDClient,
    vault: string
): Promise<void> {
    process.stdout.write("Test - Query Vault Token Info\n");
    const result: any = await terra.wasm.contractQuery(vault, {
        token_info: {},
    });

    console.log(result);
    console.log(chalk.green(" Passed!"));
}
