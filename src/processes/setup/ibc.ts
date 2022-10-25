// LocalJuno-related imports
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { LCDClient, LocalTerra, MnemonicKey, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";

import { localjuno } from "../../config/localjunoConstants";
import { wasm_path } from "../../config/wasmPaths";
import { getWalletAddress, instantiateContract, storeCode } from "../../utils/juno/helpers";

// LocalTerra-related imports
import { instantiateContract as tInstantiateContract, storeCode as tStoreCode } from "../../utils/terra/helpers";
import { localterra } from "../../config/localterraConstants";

// IBC-related imports
import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";
import { Link, testutils, IbcClient, Logger } from "@confio/relayer";
import { ChainDefinition, SigningOpts } from "@confio/relayer/build/lib/helpers";
const { signingClient, fundAccount } = testutils;
import { stringToPath } from "@cosmjs/crypto";

import { localibc } from "../../config/localIbcConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let junoIbcClient: DirectSecp256k1HdWallet;

let icaController: string;
let icaControllerPort: string;
let icaHost: string;
let icaHostPort: string;

let terra: LocalTerra | LCDClient;
let terraIbcClient: Wallet;

let terraIcaController: string;
let terraIcaControllerPort: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

type FundingOpts = SigningOpts & {
    readonly faucet: {
        readonly mnemonic: string;
    };
};
const junod = {
    tendermintUrlWs: 'ws://localhost:26657',
    tendermintUrlHttp: 'http://localhost:26657',
    chainId: 'localjuno',
    prefix: 'juno',
    denomStaking: 'ujunox',
    denomFee: 'ujuno',
    minFee: '0.025ujuno',
    blockTime: 250,
    faucet: {
        mnemonic: localibc.mnemonicKeys.junoIbcClient,
        pubkey0: {
            type: 'tendermint/PubKeySecp256k1',
            value: 'A9cXhWb8ZpqCzkA8dQCPV29KdeRLV3rUYxrkHudLbQtS',
        },
        address0: 'juno1n8y753tnrv75dlmlnyex4h9k84jrmejycc3rxy',
    },
    ics20Port: 'transfer',
    estimatedBlockTime: 400,
    estimatedIndexerTime: 80,
};
const terrad = {
    tendermintUrlWs: 'ws://localhost:26557',
    tendermintUrlHttp: 'http://localhost:26557',
    chainId: 'localterra',
    prefix: 'terra',
    denomStaking: 'uluna',
    denomFee: 'uluna',
    minFee: '0.25uluna',
    blockTime: 250,
    faucet: {
        mnemonic: localibc.mnemonicKeys.terraIbcClient,
        pubkey0: {
            type: 'tendermint/PubKeySecp256k1',
            value: 'A0d/GxY+UALE+miWJP0qyq4/EayG1G6tsg24v+cbD6By',
        },
        address0: 'terra10ldxyk6vcupuxlugnec2ugyddy4558062cc0y9',
    },
    ics20Port: 'transfer',
    estimatedBlockTime: 400,
    estimatedIndexerTime: 80,
};
const IbcVersion = "ica-vaults-v1"; // "simple-ica-v2";

// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------
export async function setupIBC(
    juno_config: {
        juno: SigningCosmWasmClient,
        junoIbcClient: DirectSecp256k1HdWallet,
    },
    terra_config: {
        terra: LocalTerra | LCDClient,
        terraIbcClient: Wallet,
    }
): Promise<void> {
    juno = juno_config.juno;
    junoIbcClient = juno_config.junoIbcClient;
    await deployJunoIcaContracts();

    terra = terra_config.terra;
    terraIbcClient = terra_config.terraIbcClient;
    await deployTerraIcaContracts();

    await customConnSetup(junod, terrad);

}

// Clone of original "@confio/relayer/testutils/setup" util
//
// create a connection and channel for simple-ica
async function customConnSetup(srcConfig: ChainDefinition, destConfig: ChainDefinition, logger?: Logger) {

    // create apps and fund an account
    const mnemonic = localibc.mnemonicKeys.signingClient;

    const src = await customSigningClient(srcConfig, mnemonic);
    const dest = await customSigningClient(destConfig, mnemonic);

    await customFundAccount(destConfig, dest.senderAddress, '4000000');
    await customFundAccount(srcConfig, src.senderAddress, '4000000');

    const link = await Link.createWithNewConnections(src, dest);
    console.log("LINK::::", link);
    const simpleChannel = await link.createChannel("A", icaControllerPort, terraIcaHostPort, Order.ORDER_UNORDERED, IbcVersion);
    console.log("Simple Channel: ", simpleChannel);

    // also create a ics20 channel on this connection
    const ics20Info = await link.createChannel("A", junod.ics20Port, terrad.ics20Port, Order.ORDER_UNORDERED, "ics20-1");
    const ics20 = {
        juno: ics20Info.src.channelId,
        terra: ics20Info.dest.channelId,
    };
    console.log("ICS20 info: ", ics20);
}

function extras() {
    const extras = {
        // This is just for tests - don't add this in production code
        broadcastPollIntervalMs: 300,
        broadcastTimeoutMs: 5000,
    };
    return extras;
}

async function customSigningClient(opts: SigningOpts, mnemonic: string, logger?: Logger): Promise<IbcClient> {
    let signer: DirectSecp256k1HdWallet;
    if (opts.prefix == "terra") {
        signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
            hdPaths: [stringToPath("m/44'/330'/0'/0/0")]
        })
    } else {
        signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
        });
    }
    const { address } = (await signer.getAccounts())[0];
    const options = {
        prefix: opts.prefix,
        gasPrice: GasPrice.fromString(opts.minFee),
        logger,
        estimatedBlockTime: opts.estimatedBlockTime,
        estimatedIndexerTime: opts.estimatedIndexerTime,
        ...extras(),
    };
    const client = await IbcClient.connectWithSigner(opts.tendermintUrlHttp, signer, address, options);
    return client;
}

async function customFundAccount(opts: FundingOpts, rcpt: string, amount: string) {
    const client = await customSigningClient(opts, opts.faucet.mnemonic);
    const feeTokens = {
        amount,
        denom: GasPrice.fromString(opts.minFee).denom,
    };
    await client.sendTokens(rcpt, [feeTokens]);
}

async function deployJunoIcaContracts(): Promise<void> {
    const junoIbcClientAddr = await getWalletAddress(junoIbcClient);
    // Step 1: Upload the wasms
    process.stdout.write("Uploading ica_controller wasm");
    const icaControllerCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.core}/ica_controller.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaControllerCodeId}`);

    process.stdout.write("Uploading ica_host wasm");
    const icaHostCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.core}/ica_host.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaHostCodeId}`);

    process.stdout.write("Uploading cw1_whitelist wasm");
    const cw1WhitelistCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.cosmwasm}/cw1_whitelist.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`);


    // Step 2: JunoIbcClient set up the "ica"-related contracts
    process.stdout.write("Instantiating ica_controller contract");
    const icaControllerResult = await instantiateContract(juno, junoIbcClientAddr, junoIbcClientAddr, icaControllerCodeId, {});
    icaController = icaControllerResult.contractAddress as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_controller contractAddress")}=${icaController}`);

    const icaControllerContract = await juno.getContract(icaController);
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_controller ibcPortId")}=${icaControllerContract.ibcPortId}`);
    icaControllerPort = icaControllerContract.ibcPortId!;

    process.stdout.write("Instantiating ica_host contract");
    const icaHostResult = await instantiateContract(juno, junoIbcClientAddr, junoIbcClientAddr, icaHostCodeId, {
        cw1_code_id: cw1WhitelistCodeId,
    });
    icaHost = icaHostResult.contractAddress as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host contractAddress")}=${icaHost}`);

    const icaHostContract = await juno.getContract(icaHost);
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host ibcPortId")}=${icaHostContract.ibcPortId}`);
    icaHostPort = icaHostContract.ibcPortId!;

    console.log(chalk.green(" Done!"));
}

async function deployTerraIcaContracts(): Promise<void> {
    // Step 1: Upload the wasms
    process.stdout.write("Uploading ica_controller wasm...");
    const icaControllerCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.core}/ica_controller.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaControllerCodeId}`);

    process.stdout.write("Uploading ica_host wasm...");
    const icaHostCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.core}/ica_host.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaHostCodeId}`);

    process.stdout.write("Uploading cw1_whitelist wasm...");
    const cw1WhitelistCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.cosmwasm}/cw1_whitelist.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`);

    // Step 2: TerraIbcClient set up the "ica"-related contracts
    process.stdout.write("Instantiating ica_controller contract");
    const icaControllerResult = await tInstantiateContract(terra, terraIbcClient, terraIbcClient, icaControllerCodeId, {});
    terraIcaController = icaControllerResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_controller contractAddress")}=${terraIcaController}`);

    const icaControllerContract = await terra.wasm.contractInfo(terraIcaController);
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_controller ibcPortId")}=${icaControllerContract.ibc_port_id!}`);
    terraIcaControllerPort = icaControllerContract.ibc_port_id!;

    process.stdout.write("Instantiating ica_host contract");
    const icaHostResult = await tInstantiateContract(terra, terraIbcClient, terraIbcClient, icaHostCodeId, {
        cw1_code_id: cw1WhitelistCodeId,
    });
    terraIcaHost = icaHostResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host contractAddress")}=${terraIcaHost}`);

    const icaHostContract = await terra.wasm.contractInfo(terraIcaHost);
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host ibcPortId")}=${icaHostContract.ibc_port_id!}`);
    terraIcaHostPort = icaHostContract.ibc_port_id!;

    console.log(chalk.green(" Done!"));
}