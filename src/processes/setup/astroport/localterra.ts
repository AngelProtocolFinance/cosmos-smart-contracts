/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";

import { storeCode, instantiateContract, sendTransaction } from "../../../utils/terra/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { localterra } from "../../../config/localterraConstants";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------

let terra: LocalTerra | LCDClient;
let apTeam: Wallet;
let apTeam2: Wallet;
let apTeam3: Wallet;

let astro: string;
let astroportGenerator: string;
let astroportFactory: string;
let astroportRouter: string;
let usdcUsdtPair: string;



// -------------------------------------------------------------------------------------
// setup all contracts for LocalTerra and TestNet
// -------------------------------------------------------------------------------------
export async function setupAstroportPlatform(
    _terra: LocalTerra | LCDClient,
    wallets: {
        apTeam: Wallet,
        apTeam2: Wallet,
        apTeam3: Wallet,
    }
): Promise<void> {
    terra = _terra;
    apTeam = wallets.apTeam;
    apTeam2 = wallets.apTeam2;
    apTeam3 = wallets.apTeam3;

    await setup(terra, apTeam);

    console.log(chalk.green(" Done!"));
    process.exit();
}


async function setup(
    terra: LocalTerra | LCDClient,
    apTeam: Wallet,
): Promise<void> {
    // Step 1. Upload all local wasm files and capture the codes for each....

    process.stdout.write("Uploading Astroport Token Wasm");
    const tokenCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_token.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${tokenCodeId}`);

    process.stdout.write("Uploading Astroport Pair Wasm");
    const pairCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_pair_stable.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${pairCodeId}`);

    process.stdout.write("Uploading Astroport Factory Wasm");
    const factoryCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_factory.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${factoryCodeId}`);

    process.stdout.write("Uploading Astroport Router Wasm");
    const routerCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_router.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${routerCodeId}`);

    process.stdout.write("Uploading Astroport Generator Wasm");
    const generatorCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_generator.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${generatorCodeId}`);

    process.stdout.write("Uploading Astroport Whitelist Wasm");
    const whitelistCodeId = await storeCode(
        terra,
        apTeam,
        `${wasm_path.astroport}/astroport_whitelist.wasm`
    );
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${whitelistCodeId}`);

    // Step 2. Instantiate contracts

    // ASTRO token 
    process.stdout.write("Instantiating ASTRO token contract");

    const astroTokenResult = await instantiateContract(
        terra,
        apTeam,
        apTeam,
        tokenCodeId,
        {
            "name": "Test ASTRO TOKEN",
            "symbol": "ASTRO",
            "decimals": 6,
            "initial_balances": [
                {
                    "address": apTeam.key.accAddress,
                    "amount": localterra.astroport.initial_ASTRO_supply,
                }
            ]
        }
    );
    astro = astroTokenResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astro}`);

    // AstroportFactory
    process.stdout.write("Instantiating Astroport Factory contract");
    const astroportFactoryResult = await instantiateContract(
        terra,
        apTeam,
        apTeam,
        factoryCodeId,
        {
            "token_code_id": tokenCodeId,
            "fee_address": undefined,
            "owner": apTeam.key.accAddress,
            "generator_address": undefined,
            "whitelist_code_id": whitelistCodeId,
            "pair_configs": [{
                "code_id": pairCodeId,
                "pair_type": {
                    "stable": {}
                },
                "total_fee_bps": 0,
                "maker_fee_bps": 0,
                "is_disabled": false,
                "is_generator_disabled": false,
            }]
        }
    );
    astroportFactory = astroportFactoryResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;

    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroportFactory}`);


    // Astroport Generator
    //  `tokens_per_block`: "0"  -> mock value
    //  `start_block`: 0         -> mock value
    //  `vesting_contract`: apTeam.key.accAddress   -> mock value
    process.stdout.write("Instantiating Astroport Generator contract");
    const astroportGeneratorResult = await instantiateContract(
        terra,
        apTeam,
        apTeam,
        generatorCodeId,
        {
            "owner": apTeam.key.accAddress,
            "factory": astroportFactory,
            "generator_controller": undefined,
            "voting_escrow_delegation": undefined,
            "voting_escrow": undefined,
            "guardian": undefined,
            "astro_token": astro,
            "tokens_per_block": "0",
            "start_block": 0,
            "allowed_reward_proxies": [],
            "vesting_contract": apTeam.key.accAddress,
            "whitelist_code_id": whitelistCodeId,
        }
    );

    astroportGenerator = astroportGeneratorResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;

    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroportGenerator}`);

    // Astroport Router
    process.stdout.write("Instantiating Astroport Router contract");
    const astroportRouterResult = await instantiateContract(
        terra,
        apTeam,
        apTeam,
        routerCodeId,
        {
            astroport_factory: astroportFactory,
        }
    );

    astroportRouter = astroportRouterResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;

    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroportRouter}`);

    // Usdc-Usdt pair
    process.stdout.write("Instantiating USDC-USDT Swap(Pair) contract");

    const usdcUsdtPairResult = await sendTransaction(
        terra,
        apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            astroportFactory,
            {
                "create_pair": {
                    "pair_type": {
                        "stable": {}
                    },
                    "asset_infos": [
                        {
                            "native_token": {
                                "denom": "usdc",
                            }
                        },
                        {
                            "native_token": {
                                "denom": "usdt"
                            }
                        }
                    ],
                    "init_params": undefined,
                }
            },
        )
    ]
    );

    usdcUsdtPair = usdcUsdtPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${usdcUsdtPair}`);
    console.log("logs: ", usdcUsdtPairResult.logs[0].events);

    // Step 3. Update the contracts' configuration

    // Update configuration of contracts
    process.stdout.write("Update Astroport-factory config");
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            astroportFactory,
            {
                update_config: {
                    generator_address: astroportGenerator,
                    token_code_id: undefined,
                    fee_address: undefined,
                    whitelist_code_id: undefined,
                }
            }
        )
    ]);
    console.log(chalk.green(" Done!"));
}