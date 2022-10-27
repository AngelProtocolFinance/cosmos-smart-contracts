// LocalJuno-related imports
import { SigningCosmWasmClient, toBinary } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { junod, terrad } from "../../config/localIbcConstants";
import { localjuno } from "../../config/localjunoConstants";
import { localterra } from "../../config/localterraConstants";

import { wasm_path } from "../../config/wasmPaths";
import { customSigningCosmWasmClient } from "../../utils/ibc";
import { getWalletAddress, instantiateContract, sendMessageViaCw3Proposal, storeCode } from "../../utils/juno/helpers";

// LocalTerra-related imports
import { instantiateContract as tInstantiateContract, storeCode as tStoreCode, sendTransaction as tSendTransaction } from "../../utils/terra/helpers";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let junoIbcClient: DirectSecp256k1HdWallet;
let junoIbcClientAddr: string;

let junoIcaController: string;
let junoIcaControllerPort: string;
let junoIcaHost: string;
let junoIcaHostPort: string;

let terra: LocalTerra | LCDClient;
let terraIbcClient: Wallet;

let terraIcaController: string;
let terraIcaControllerPort: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

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
    junoIbcClientAddr = await getWalletAddress(junoIbcClient);
    await deployJunoIcaContracts();

    terra = terra_config.terra;
    terraIbcClient = terra_config.terraIbcClient;
    await deployTerraIcaContracts();

    await postProcess();
    console.log(chalk.green(" Done!"));
}

async function deployJunoIcaContracts(): Promise<void> {
    // Step 1: Upload the wasms
    process.stdout.write("Uploading ica_controller wasm on JUNO");
    const icaControllerCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.core}/ica_controller.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaControllerCodeId}`);

    process.stdout.write("Uploading ica_host wasm on JUNO");
    const icaHostCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.core}/ica_host.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaHostCodeId}`);

    process.stdout.write("Uploading cw1_whitelist wasm on JUNO");
    const cw1WhitelistCodeId = await storeCode(juno, junoIbcClientAddr, `${wasm_path.cosmwasm}/cw1_whitelist.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`);


    // Step 2: JunoIbcClient set up the "ica"-related contracts
    process.stdout.write("Instantiating (juno)ica_controller contract");
    const icaControllerResult = await instantiateContract(juno, junoIbcClientAddr, junoIbcClientAddr, icaControllerCodeId, {});
    junoIcaController = icaControllerResult.contractAddress as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("(juno)ica_controller contractAddress")}=${junoIcaController}`);

    process.stdout.write("Querying (juno)ica_controller ibcPort");
    const icaControllerContract = await juno.getContract(junoIcaController);
    junoIcaControllerPort = icaControllerContract.ibcPortId!;
    console.log(chalk.green(" Done!"), `${chalk.blue("(juno)ica_controller ibcPortId")}=${junoIcaControllerPort}`);

    process.stdout.write("Instantiating (juno)ica_host contract");
    const icaHostResult = await instantiateContract(juno, junoIbcClientAddr, junoIbcClientAddr, icaHostCodeId, {
        cw1_code_id: cw1WhitelistCodeId,
    });
    process.stdout.write("Querying (juno)ica_host ibcPort");
    junoIcaHost = icaHostResult.contractAddress as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("(juno)ica_host contractAddress")}=${junoIcaHost}`);

    const icaHostContract = await juno.getContract(junoIcaHost);
    junoIcaHostPort = icaHostContract.ibcPortId!;
    console.log(chalk.green(" Done!"), `${chalk.blue("(juno)ica_host ibcPortId")}=${junoIcaHostPort}`);
}

async function deployTerraIcaContracts(): Promise<void> {
    // Step 1: Upload the wasms
    process.stdout.write("Uploading ica_controller wasm on TERRA");
    const icaControllerCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.core}/ica_controller.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaControllerCodeId}`);

    process.stdout.write("Uploading ica_host wasm on TERRA");
    const icaHostCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.core}/ica_host.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${icaHostCodeId}`);

    process.stdout.write("Uploading cw1_whitelist wasm on TERRA");
    const cw1WhitelistCodeId = await tStoreCode(terra, terraIbcClient, `${wasm_path.cosmwasm}/cw1_whitelist.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`);

    // Step 2: TerraIbcClient set up the "ica"-related contracts
    process.stdout.write("Instantiating (terra)ica_controller contract");
    const icaControllerResult = await tInstantiateContract(terra, terraIbcClient, terraIbcClient, icaControllerCodeId, {});
    terraIcaController = icaControllerResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("(terra)ica_controller contractAddress")}=${terraIcaController}`);

    process.stdout.write("Querying (terra)ica_controller ibcPort");
    const icaControllerContract = await terra.wasm.contractInfo(terraIcaController);
    terraIcaControllerPort = icaControllerContract.ibc_port_id!;
    console.log(chalk.green(" Done!"), `${chalk.blue("(terra)ica_controller ibcPortId")}=${terraIcaControllerPort}`);


    process.stdout.write("Instantiating (terra)ica_host contract");
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
    console.log(chalk.green(" Done!"), `${chalk.blue("(terra)ica_host contractAddress")}=${terraIcaHost}`);

    process.stdout.write("Querying (terra)ica_host ibcPort");
    const icaHostContract = await terra.wasm.contractInfo(terraIcaHost);
    terraIcaHostPort = icaHostContract.ibc_port_id!;
    console.log(chalk.green(" Done!"), `${chalk.blue("(terra)ica_host ibcPortId")}=${terraIcaHostPort}`);
}

async function postProcess() {
    process.stdout.write("Updating admins of controller & host contracts");
    await juno.execute(junoIbcClientAddr, junoIcaController, {
        update_admin: {
            admin: localjuno.contracts.accounts,
        }
    }, "auto");
    // await juno.execute(junoIbcClientAddr, junoIcaHost, {
    //     update_admin: {
    //         admin: localjuno.contracts.accounts,
    //     }
    // }, "auto");

    await tSendTransaction(terra, terraIbcClient, [
        new MsgExecuteContract(
            terraIbcClient.key.accAddress,
            terraIcaController,
            {
                update_admin: {
                    admin: localterra.contracts.vaultLocked1,
                }
            }
        )
    ]);
    // await tSendTransaction(terra, terraIbcClient, [
    //     new MsgExecuteContract(
    //         terraIbcClient.key.accAddress,
    //         terraIcaHost,
    //         {
    //             update_admin: {
    //                 admin: localterra.contracts.vaultLocked1,
    //             }
    //         }
    //     )
    // ]);

    process.stdout.write("Updating configs of `(juno) accounts` and `(terra) vault` contracts");
    const junoAPTeamSigner = await customSigningCosmWasmClient(junod, localjuno.mnemonicKeys.apTeam);
    await sendMessageViaCw3Proposal(junoAPTeamSigner.sign, junoAPTeamSigner.senderAddress, localjuno.contracts.cw3ApTeam, localjuno.contracts.accounts, {
        update_config: {
            new_registrar: localjuno.contracts.registrar,
            max_general_category_id: 1,
            ibc_controller: junoIcaController,
        }
    });

    const terraApTeamSigner = await customSigningCosmWasmClient(terrad, localjuno.mnemonicKeys.apTeam);
    await terraApTeamSigner.sign.execute(terraApTeamSigner.senderAddress, localterra.contracts.vaultLocked1, {
        update_config: {
            ibc_host: terraIcaHost,
            ibc_controller: terraIcaController,
        }
    }, "auto");
    await terraApTeamSigner.sign.execute(terraApTeamSigner.senderAddress, localterra.contracts.vaultLiquid1, {
        update_config: {
            ibc_host: terraIcaHost,
            ibc_controller: terraIcaController,
        }
    }, "auto");
}