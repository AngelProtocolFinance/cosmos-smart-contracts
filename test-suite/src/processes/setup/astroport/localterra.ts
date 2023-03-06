/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";

import { storeCode, instantiateContract, sendTransaction, toEncodedBinary } from "../../../utils/terra/helpers";
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
let usdcUsdtPairLpToken: string;
let lunaAstroPair: string;
let lunaAstroPairLpToken: string;
let lunaUsdcPair: string;
let lunaUsdcPairLpToken: string;
let lunaUsdtPair: string;
let lunaUsdtPairLpToken: string;
let astroUsdcPair: string;
let astroUsdcPairLpToken: string;

// -------------------------------------------------------------------------------------
// setup all contracts for LocalTerra and TestNet
// -------------------------------------------------------------------------------------
export async function setupAstroportPlatform(
    _terra: LocalTerra | LCDClient,
    wallets: {
        apTeam: Wallet;
        apTeam2: Wallet;
        apTeam3: Wallet;
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

async function setup(terra: LocalTerra | LCDClient, apTeam: Wallet): Promise<void> {
    // Step 1. Upload all local wasm files and capture the codes for each....

    process.stdout.write("Uploading Astroport Token Wasm");
    const tokenCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_token.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${tokenCodeId}`);

    process.stdout.write("Uploading Astroport Pair Wasm");
    const pairCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_pair.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${pairCodeId}`);

    process.stdout.write("Uploading Astroport Stable Pair Wasm");
    const stablePairCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_pair_stable.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${stablePairCodeId}`);

    process.stdout.write("Uploading Astroport Factory Wasm");
    const factoryCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_factory.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${factoryCodeId}`);

    process.stdout.write("Uploading Astroport Router Wasm");
    const routerCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_router.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${routerCodeId}`);

    process.stdout.write("Uploading Astroport Generator Wasm");
    const generatorCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_generator.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${generatorCodeId}`);

    process.stdout.write("Uploading Astroport Whitelist Wasm");
    const whitelistCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_whitelist.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${whitelistCodeId}`);

    process.stdout.write("Uploading Astroport Vesting Wasm");
    const vestingCodeId = await storeCode(terra, apTeam, `${wasm_path.astroport}/astroport_vesting.wasm`);
    console.log(chalk.green(" Done!"), `${chalk.blue("codeId")} = ${vestingCodeId}`);

    // Step 2. Instantiate contracts

    // ASTRO token
    process.stdout.write("Instantiating ASTRO token contract");
    const astroTokenResult = await instantiateContract(terra, apTeam, apTeam, tokenCodeId, {
        name: "Test ASTRO TOKEN",
        symbol: "ASTRO",
        decimals: 6,
        initial_balances: [
            {
                address: apTeam.key.accAddress,
                amount: localterra.astroport.initial_ASTRO_supply,
            },
        ],
    });
    astro = astroTokenResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astro}\n`);

    // AstroportFactory
    process.stdout.write("Instantiating Astroport Factory contract");
    const astroportFactoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
        token_code_id: tokenCodeId,
        fee_address: undefined,
        owner: apTeam.key.accAddress,
        generator_address: undefined,
        whitelist_code_id: whitelistCodeId,
        pair_configs: [
            {
                code_id: pairCodeId,
                pair_type: {
                    xyk: {},
                },
                total_fee_bps: 0,
                maker_fee_bps: 0,
                is_disabled: false,
                is_generator_disabled: false,
            },
            {
                code_id: stablePairCodeId,
                pair_type: {
                    stable: {},
                },
                total_fee_bps: 0,
                maker_fee_bps: 0,
                is_disabled: false,
                is_generator_disabled: false,
            },
        ],
    });
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
    const astroportVestingResult = await instantiateContract(terra, apTeam, apTeam, vestingCodeId, {
        owner: apTeam.key.accAddress,
        token_addr: astro,
    });
    const astroportVesting = astroportVestingResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;

    process.stdout.write("Instantiating Astroport Generator contract");
    const astroportGeneratorResult = await instantiateContract(terra, apTeam, apTeam, generatorCodeId, {
        owner: apTeam.key.accAddress,
        allowed_reward_proxies: [],
        astro_token: astro,
        start_block: "0",
        tokens_per_block: "1000",
        vesting_contract: astroportVesting,

        factory: astroportFactory,
        generator_controller: undefined,
        voting_escrow: undefined,
        guardian: undefined,
        whitelist_code_id: whitelistCodeId,
    });

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
    const astroportRouterResult = await instantiateContract(terra, apTeam, apTeam, routerCodeId, {
        astroport_factory: astroportFactory,
    });

    astroportRouter = astroportRouterResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
        })?.value as string;

    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroportRouter}\n`);

    // Usdc-Usdt pair
    process.stdout.write("Instantiating USDC-USDT Swap(Pair) contract");
    const usdcUsdtPairResult = await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            create_pair: {
                pair_type: {
                    stable: {},
                },
                asset_infos: [
                    {
                        native_token: {
                            denom: localterra.denoms.usdc,
                        },
                    },
                    {
                        native_token: {
                            denom: localterra.denoms.usdt,
                        },
                    },
                ],
                init_params: toEncodedBinary({ amp: 1 }),
            },
        }),
    ]);

    usdcUsdtPair = usdcUsdtPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
            // return event.type == "wasm";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${usdcUsdtPair}`);

    // Get the LP Token address of newly created pair
    process.stdout.write("Query new USDC/USDT pair's LP Token contract");
    let res: any = await terra.wasm.contractQuery(usdcUsdtPair, { pair: {} });
    usdcUsdtPairLpToken = res.liquidity_token as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${usdcUsdtPairLpToken}\n`);

    // Send liquidity to USDC/USDT pair for swaps
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            usdcUsdtPair,
            {
                provide_liquidity: {
                    assets: [
                        {
                            info: {
                                native_token: { denom: localterra.denoms.usdc },
                            },
                            amount: localterra.astroport.usdc_usdt_pair_usdc_liquidity,
                        },
                        {
                            info: {
                                native_token: { denom: localterra.denoms.usdt },
                            },
                            amount: localterra.astroport.usdc_usdt_pair_usdt_liquidity,
                        },
                    ],
                    slippage_tolerance: undefined,
                    auto_stake: undefined, // Option<bool> ?
                    receiver: undefined,
                },
            },
            {
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4":
                    localterra.astroport.usdc_usdt_pair_usdc_liquidity,
                "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF":
                    localterra.astroport.usdc_usdt_pair_usdt_liquidity,
            }
        ),
    ]);

    // Luna-ASTRO pair
    process.stdout.write("Instantiating Luna-ASTRO Swap(Pair) contract");
    const lunaAstroPairResult = await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            create_pair: {
                pair_type: {
                    xyk: {},
                },
                asset_infos: [
                    {
                        native_token: {
                            denom: "uluna",
                        },
                    },
                    {
                        token: {
                            contract_addr: astro,
                        },
                    },
                ],
                init_params: undefined,
            },
        }),
    ]);

    lunaAstroPair = lunaAstroPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
            // return event.type == "wasm";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaAstroPair}`);

    // Get the LP Token address of newly created pair
    process.stdout.write("Query new Luna-ASTRO pair's LP Token contract");
    res = await terra.wasm.contractQuery(lunaAstroPair, { pair: {} });
    lunaAstroPairLpToken = res.liquidity_token as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaAstroPairLpToken}\n`);

    // Send liquidity to Luna/ASTRO pair for swaps
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astro, {
            increase_allowance: {
                amount: "100000000000",
                spender: lunaAstroPair,
            },
        }),
    ]);
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            lunaAstroPair,
            {
                provide_liquidity: {
                    assets: [
                        {
                            info: {
                                native_token: { denom: "uluna" },
                            },
                            amount: "100000000000",
                        },
                        {
                            info: {
                                token: { contract_addr: astro },
                            },
                            amount: "100000000000",
                        },
                    ],
                    slippage_tolerance: undefined,
                    auto_stake: undefined, // Option<bool> ?
                    receiver: undefined,
                },
            },
            {
                uluna: "100000000000",
            }
        ),
    ]);

    // Luna-USDC pair
    process.stdout.write("Instantiating Luna-USDC Swap(Pair) contract");
    const lunaUsdcPairResult = await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            create_pair: {
                pair_type: {
                    xyk: {},
                },
                asset_infos: [
                    {
                        native_token: {
                            denom: "uluna",
                        },
                    },
                    {
                        native_token: {
                            denom: localterra.denoms.usdc,
                        },
                    },
                ],
                init_params: undefined,
            },
        }),
    ]);

    lunaUsdcPair = lunaUsdcPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
            // return event.type == "wasm";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaUsdcPair}`);

    // Get the LP Token address of newly created pair
    process.stdout.write("Query new Luna/USDC pair's LP Token contract");
    res = await terra.wasm.contractQuery(lunaUsdcPair, { pair: {} });
    lunaUsdcPairLpToken = res.liquidity_token as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaUsdcPairLpToken}\n`);

    // Send liquidity to Luna/USDC pair for swaps
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            lunaUsdcPair,
            {
                provide_liquidity: {
                    assets: [
                        {
                            info: {
                                native_token: { denom: "uluna" },
                            },
                            amount: "100000000000",
                        },
                        {
                            info: {
                                native_token: { denom: localterra.denoms.usdc },
                            },
                            amount: "100000000000",
                        },
                    ],
                    slippage_tolerance: undefined,
                    auto_stake: undefined, // Option<bool> ?
                    receiver: undefined,
                },
            },
            {
                uluna: "100000000000",
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4": "100000000000",
            }
        ),
    ]);

    // Luna-USDT pair
    process.stdout.write("Instantiating Luna-USDT Swap(Pair) contract");
    const lunaUsdtPairResult = await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            create_pair: {
                pair_type: {
                    xyk: {},
                },
                asset_infos: [
                    {
                        native_token: {
                            denom: "uluna",
                        },
                    },
                    {
                        native_token: {
                            denom: localterra.denoms.usdt,
                        },
                    },
                ],
                init_params: undefined,
            },
        }),
    ]);

    lunaUsdtPair = lunaUsdtPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
            // return event.type == "wasm";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaUsdtPair}`);

    // Get the LP Token address of newly created pair
    process.stdout.write("Query new Luna/USDT pair's LP Token contract");
    res = await terra.wasm.contractQuery(lunaUsdtPair, { pair: {} });
    lunaUsdtPairLpToken = res.liquidity_token as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${lunaUsdtPairLpToken}\n`);

    // Send liquidity to Luna/USDC pair for swaps
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            lunaUsdtPair,
            {
                provide_liquidity: {
                    assets: [
                        {
                            info: {
                                native_token: { denom: "uluna" },
                            },
                            amount: "100000000000",
                        },
                        {
                            info: {
                                native_token: { denom: localterra.denoms.usdt },
                            },
                            amount: "100000000000",
                        },
                    ],
                    slippage_tolerance: undefined,
                    auto_stake: undefined, // Option<bool> ?
                    receiver: undefined,
                },
            },
            {
                uluna: "100000000000",
                "ibc/CBF67A2BCF6CAE343FDF251E510C8E18C361FC02B23430C121116E0811835DEF": "100000000000",
            }
        ),
    ]);

    // ASTRO-USDC pair
    process.stdout.write("Instantiating Astro-USDC Swap(Pair) contract");
    const astroUsdcPairResult = await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            create_pair: {
                pair_type: {
                    xyk: {},
                },
                asset_infos: [
                    {
                        token: {
                            contract_addr: astro,
                        },
                    },
                    {
                        native_token: {
                            denom: localterra.denoms.usdc,
                        },
                    },
                ],
                init_params: undefined,
            },
        }),
    ]);

    astroUsdcPair = astroUsdcPairResult.logs[0].events
        .find((event) => {
            return event.type == "instantiate";
            // return event.type == "wasm";
        })
        ?.attributes.find((attribute) => {
            return attribute.key == "_contract_address";
            // return attribute.key == "pair_contract_addr";
        })?.value as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroUsdcPair}`);

    // Get the LP Token address of newly created pair
    process.stdout.write("Query new ASTRO/USDC pair's LP Token contract");
    res = await terra.wasm.contractQuery(astroUsdcPair, { pair: {} });
    astroUsdcPairLpToken = res.liquidity_token as string;
    console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${astroUsdcPairLpToken}\n`);

    // Send liquidity to ASTRO/USDC pair for swaps
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astro, {
            increase_allowance: {
                amount: "100000000000",
                spender: astroUsdcPair,
            },
        }),
    ]);
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(
            apTeam.key.accAddress,
            astroUsdcPair,
            {
                provide_liquidity: {
                    assets: [
                        {
                            info: {
                                native_token: { denom: localterra.denoms.usdc },
                            },
                            amount: "100000000000",
                        },
                        {
                            info: {
                                token: { contract_addr: astro },
                            },
                            amount: "100000000000",
                        },
                    ],
                    slippage_tolerance: undefined,
                    auto_stake: undefined, // Option<bool> ?
                    receiver: undefined,
                },
            },
            {
                "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4": "100000000000",
            }
        ),
    ]);

    // Step 3. Update the contracts' configuration

    // Update configuration of contracts
    process.stdout.write("Update Astroport-factory config");
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportFactory, {
            update_config: {
                generator_address: astroportGenerator,
                token_code_id: undefined,
                fee_address: undefined,
                whitelist_code_id: undefined,
            },
        }),
    ]);
    console.log(chalk.green(" Done!"));

    // Setup pools in `astroport_generator` contract
    process.stdout.write("Setup pool for USDC-USDT pair in Generator contract");
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astroportGenerator, {
            setup_pools: {
                pools: [[usdcUsdtPairLpToken, "100000000000"]],
            },
        }),
    ]);

    // Setup vesting schedule in vesting contract
    process.stdout.write("Setup vesting schedule in vesting contract");
    await sendTransaction(terra, apTeam, [
        new MsgExecuteContract(apTeam.key.accAddress, astro, {
            increase_allowance: {
                spender: astroportVesting,
                amount: "100000000000000",
                expires: undefined,
            },
        }),
        new MsgExecuteContract(apTeam.key.accAddress, astro, {
            send: {
                msg: toEncodedBinary({
                    register_vesting_accounts: {
                        vesting_accounts: [
                            {
                                address: astroportGenerator,
                                schedules: [
                                    {
                                        start_point: { time: 1654599600, amount: "1000000000000" },
                                        end_point: { time: 1686135600, amount: "100000000000000" },
                                    },
                                ],
                            },
                        ],
                    },
                }),
                amount: "100000000000000",
                contract: astroportVesting,
            },
        }),
    ]);

    console.log(chalk.green(" Done!"));
}
