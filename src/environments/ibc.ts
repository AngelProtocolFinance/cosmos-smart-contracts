import chalk from "chalk";

import { localibc } from "../config/localIbcConstants";
import { setupIBC } from "../processes/setup/ibc";
import { testExecuteIBC } from "../processes/tests/ibc";


// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let junoIcaController: string;
let junoIcaHost: string;

let terraIcaController1: string;
let terraIcaController2: string;
let terraIcaHost: string;

// -------------------------------------------------------------------------------------
// initialize variables
// -------------------------------------------------------------------------------------
async function initialize() {

    junoIcaController = localibc.config.junoIcaController;
    junoIcaHost = localibc.config.junoIcaHost;

    terraIcaController1 = localibc.config.terraIcaController1;
    terraIcaController2 = localibc.config.terraIcaController2;
    terraIcaHost = localibc.config.terraIcaHost;

    console.log(`Using ${chalk.cyan(junoIcaController)} as Juno ica controller contract`);
    console.log(`Using ${chalk.cyan(`wasm.${junoIcaController}`)} as Juno ica controller Port`);
    console.log(`Using ${chalk.cyan(junoIcaHost)} as Juno ica host contract`);
    console.log(`Using ${chalk.cyan(`wasm.${junoIcaHost}`)} as Juno ica host Port`);

    console.log(`Using ${chalk.cyan(terraIcaController1)} as Terra ica controller1 contract`);
    console.log(`Using ${chalk.cyan(`wasm.${terraIcaController1}`)} as Terra ica controller1 Port`);
    console.log(`Using ${chalk.cyan(terraIcaController2)} as Terra ica controller2 contract`);
    console.log(`Using ${chalk.cyan(`wasm.${terraIcaController2}`)} as Terra ica controller2 Port`);
    console.log(`Using ${chalk.cyan(terraIcaHost)} as Terra ica host contract`);
    console.log(`Using ${chalk.cyan(`wasm.${terraIcaHost}`)} as Terra ica host Port`);
}

export async function startSetupIBC(): Promise<void> {
    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // Setup contracts
    console.log(chalk.yellow("\nStep 2. IBC Contracts Setup"));
    await setupIBC();
}

export async function startTestIBC(): Promise<void> {
    // Initialize environment information
    console.log(chalk.yellow("\nStep 1. Environment Info"));
    await initialize();

    // Tests
    await testExecuteIBC(
        {
            junoIcaController,
            junoIcaHost,
            terraIcaController1,
            terraIcaController2,
            terraIcaHost,
        }
    );
}