// LocalJuno-related imports
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";

import { localjuno } from "../../config/localjunoConstants";
import { wasm_path } from "../../config/wasmPaths";
import { getWalletAddress, instantiateContract, storeCode } from "../../utils/juno/helpers";

// LocalTerra-related imports
import { instantiateContract as tInstantiateContract, storeCode as tStoreCode } from "../../utils/terra/helpers";
import { localterra } from "../../config/localterraConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let juno: SigningCosmWasmClient;
let junoIbcClient: DirectSecp256k1HdWallet;

let icaController: string;
let icaHost: string;

let terra: LocalTerra | LCDClient;
let terraIbcClient: Wallet;

let terraIcaController: string;
let terraIcaHost: string;

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
    await createJunoIBC();

    terra = terra_config.terra;
    terraIbcClient = terra_config.terraIbcClient;
    await createTerraIBC();
}

async function createJunoIBC(): Promise<void> {
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

    process.stdout.write("Instantiating ica_host contract");
    const icaHostResult = await instantiateContract(juno, junoIbcClientAddr, junoIbcClientAddr, icaHostCodeId, {
        cw1_code_id: cw1WhitelistCodeId,
    });
    icaHost = icaHostResult.contractAddress as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host contractAddress")}=${icaHost}`);

    const icaHostContract = await juno.getContract(icaHost);
    console.log(chalk.green(" Done!"), `${chalk.blue("ica_host ibcPortId")}=${icaHostContract.ibcPortId}`);

    console.log(chalk.green(" Done!"));
}

async function createTerraIBC(): Promise<void> {
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

    console.log(chalk.green(" Done!"));
}