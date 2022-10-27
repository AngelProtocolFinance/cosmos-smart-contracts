// LocalJuno-related imports
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import chalk from "chalk";

import { localjuno } from "../config/localjunoConstants";
import { setupIBC } from "../processes/setup/ibc";
import { getWalletAddress } from "../utils/juno/helpers";

// LocalTerra-related imports
import { LCDClient, LocalTerra, MnemonicKey, MsgSend, Wallet } from "@terra-money/terra.js";
import { localterra } from "../config/localterraConstants";
import { sendTransaction } from "../utils/terra/helpers";

import { localibc } from "../config/localIbcConstants";
import { testExecuteIBC } from "../processes/tests/ibc";


// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let junoIbcClient: DirectSecp256k1HdWallet;
let junoIbcClientAccount: string;

let terra: LocalTerra | LCDClient;
let terraIbcClient: Wallet;

let junoIcaController: string;
let junoIcaControllerPort: string;
let junoIcaHost: string;
let junoIcaHostPort: string;

let terraIcaController: string;
let terraIcaControllerPort: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
async function initialize() {
    // Setup the `junoIbcClient` wallet
    junoIbcClient = await DirectSecp256k1HdWallet.fromMnemonic(localibc.mnemonicKeys.junoIbcClient, { prefix: localjuno.networkInfo.walletPrefix });
    junoIbcClientAccount = await getWalletAddress(junoIbcClient);
    juno = await SigningCosmWasmClient.connectWithSigner(localjuno.networkInfo.url, junoIbcClient, { gasPrice: GasPrice.fromString(localjuno.networkInfo.gasPrice) });

    // Fund the `junoIbcClient` wallet 
    const funder = await DirectSecp256k1HdWallet.fromMnemonic(localjuno.mnemonicKeys.apTeam, { prefix: localjuno.networkInfo.walletPrefix });
    const funderAccount = await getWalletAddress(funder);
    const funderSigner = await SigningCosmWasmClient.connectWithSigner(localjuno.networkInfo.url, funder, { gasPrice: GasPrice.fromString(localjuno.networkInfo.gasPrice) });
    const balance = await funderSigner.getBalance(junoIbcClientAccount, localjuno.networkInfo.nativeToken);
    if (parseInt(balance.amount) == 0) {
        await funderSigner.sendTokens(funderAccount, junoIbcClientAccount, [coin("100000000", localjuno.networkInfo.nativeToken)], "auto");
    }
    console.log(`Using ${chalk.cyan(junoIbcClientAccount)} as Juno IBC Client`);

    // Setup the `terraIbcClient` wallet
    terra = new LCDClient({
        URL: localterra.networkInfo.url,
        chainID: localterra.networkInfo.chainId,
    });
    terraIbcClient = new Wallet(terra, new MnemonicKey({ mnemonic: localibc.mnemonicKeys.terraIbcClient }));

    // Fund the `terraIbcClient` wallet
    const balances = await terra.bank.balance(terraIbcClient.key.accAddress);
    if (!balances[0].get("uluna")) {
        const terraFunder = new Wallet(terra, new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test10 }));
        await sendTransaction(terra, terraFunder, [
            new MsgSend(
                terraFunder.key.accAddress,
                terraIbcClient.key.accAddress,
                { uluna: "100000000" }
            )
        ]);
    }
    console.log(`Using ${chalk.cyan(terraIbcClient.key.accAddress)} as Terra IBC Client`);

    junoIcaController = localibc.config.junoIcaController;
    junoIcaControllerPort = localibc.config.junoIcaControllerPort;
    junoIcaHost = localibc.config.junoIcaHost;
    junoIcaHostPort = localibc.config.junoIcaHostPort;

    terraIcaController = localibc.config.terraIcaController;
    terraIcaControllerPort = localibc.config.terraIcaControllerPort;
    terraIcaHost = localibc.config.terraIcaHost;
    terraIcaHostPort = localibc.config.terraIcaHostPort;

    console.log(`Using ${chalk.cyan(junoIcaController)} as Juno ica controller contract`);
    console.log(`Using ${chalk.cyan(junoIcaControllerPort)} as Juno ica controller Port`);
    console.log(`Using ${chalk.cyan(junoIcaHost)} as Juno ica host contract`);
    console.log(`Using ${chalk.cyan(junoIcaHostPort)} as Juno ica host Port`);

    console.log(`Using ${chalk.cyan(terraIcaController)} as Terra ica controller contract`);
    console.log(`Using ${chalk.cyan(terraIcaControllerPort)} as Terra ica controller Port`);
    console.log(`Using ${chalk.cyan(terraIcaHost)} as Terra ica host contract`);
    console.log(`Using ${chalk.cyan(terraIcaHostPort)} as Terra ica host Port`);

}

export async function startSetupIBC(): Promise<void> {
    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // Setup contracts
    console.log(chalk.yellow("\nStep 2. IBC Contracts Setup"));
    await setupIBC(
        // juno_config
        {
            juno,
            junoIbcClient,
        },
        // terra_config
        {
            terra,
            terraIbcClient,
        }
    );
}

export async function startTestIBC(): Promise<void> {
    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // Tests
    await testExecuteIBC(
        {
            junoIcaController,
            junoIcaControllerPort,
            junoIcaHost,
            junoIcaHostPort,
            terraIcaController,
            terraIcaControllerPort,
            terraIcaHost,
            terraIcaHostPort,
        }
    );
}