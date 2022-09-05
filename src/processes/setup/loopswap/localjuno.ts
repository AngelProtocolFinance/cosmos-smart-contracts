/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { coin } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { instantiateContract, sendTransaction, sendTransactionWithFunds, storeCode } from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";

// Deploy LOOP & HALO Token and LOOP/JUNO & HALO/JUNO pair contracts to the LocalJuno
export async function setupLoopSwap(
  juno: SigningCosmWasmClient,
  apTeam: string,
  apTeam2: string,
  apTeam3: string,
  initial_loop_supply: string,
  loop_liquidity: string,
  juno_liquidity: string,
  initial_halo_supply: string,
  halo_liquidity: string,
  native_liquidity: string
): Promise<void> {
  process.stdout.write("Uploading LoopSwap Token Wasm");
  const tokenCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.loopswap}/loopswap_token.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading LoopSwap Pair Wasm");
  const pairCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.loopswap}/loopswap_pair.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading LoopSwap Factory Wasm");
  const factoryCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.loopswap}/loopswap_factory.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);


  process.stdout.write("Uploading LoopSwap Farming Wasm");
  const farmingCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.loopswap}/loopswap_farming.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${farmingCodeId}`);


  // Factory contract
  process.stdout.write("Instantiating LoopSwap Factory contract");
  const factoryResult = await instantiateContract(juno, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
  });
  const factoryContract = factoryResult.contractAddress as string;

  // Add native token decimals
  await sendTransactionWithFunds(juno, apTeam, factoryContract, {
    add_native_token_decimals: {
      denom: "ujuno",
      decimals: 6,
    },
  }, [coin(10, "ujuno")]);

  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${factoryContract}`
  );

  // LOOP token contract
  process.stdout.write("Instantiating LOOP Token contract");
  const loopTokenResult = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "LOOP Token",
    symbol: "LOOP",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_loop_supply,
      },
      {
        address: apTeam2,
        amount: initial_loop_supply,
      },
      {
        address: apTeam3,
        amount: initial_loop_supply,
      },
    ],
  });
  const loopTokenContract = loopTokenResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${loopTokenContract}`);

  // LOOP/JUNO pair contract
  process.stdout.write("Creating LOOP/JUNO pair contract ");
  const loopJunoPairResult = await sendTransaction(juno, apTeam, factoryContract, {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: loopTokenContract,
          },
        },
        {
          native_token: {
            denom: "ujuno",
          },
        },
      ],
    },
  });
  const loopJunoPairContract = loopJunoPairResult.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "pair_contract_addr";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${loopJunoPairContract}`);

  // Get the LP Token address of newly created pair
  process.stdout.write("Query new LOOP/JUNO pair's LP Token contract");
  const res: any = await juno.queryContractSmart(loopJunoPairContract, {
    pair: {},
  });
  const loopJunoPairLP = res.liquidity_token as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${loopJunoPairLP}`
  );


  // send liquidity to the new Swap pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new LOOP/JUNO pair contract @ ratio of 0.5 JUNO per LOOP"
  );
  const _liq = await sendTransaction(juno, apTeam, loopTokenContract, {
      increase_allowance: {
        amount: loop_liquidity,
        spender: loopJunoPairContract,
      },
    });
  await sendTransaction(juno, apTeam, loopJunoPairContract, {
      provide_liquidity: {
        assets: [
          {
            info: {
              token: {
                contract_addr: loopTokenContract,
              },
            },
            amount: loop_liquidity,
          },
          {
            info: {
              native_token: {
                denom: "ujuno",
              },
            },
            amount: juno_liquidity,
          },
        ],
      },
    },
    [coin(juno_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));
  

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const haloTokenResult = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "Angel Protocol Token",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_halo_supply,
      },
      {
        address: apTeam2,
        amount: initial_halo_supply,
      },
      {
        address: apTeam3,
        amount: initial_halo_supply,
      },
    ],
  });
  const haloTokenContract = haloTokenResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${haloTokenContract}`);

  // HALO/JUNO contract
  process.stdout.write("Creating HALO/JUNO pair contract ");
  const haloJunoPairResult = await sendTransaction(juno, apTeam, factoryContract, {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: haloTokenContract,
          },
        },
        {
          native_token: {
            denom: "ujuno",
          },
        },
      ],
    },
  });

  const haloJunoPairContract = haloJunoPairResult.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "pair_contract_addr";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${haloJunoPairContract}`);

  // Get the LP Token address of newly created swap pool
  process.stdout.write("Query new Swap pool's LP Token contract");
  const result: any = await juno.queryContractSmart(haloJunoPairContract, {
    pair: {},
  });
  const haloJunoPairLP = res.liquidity_token as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${haloJunoPairLP}`
  );

  // send liquidity to the new Swap pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new HALO/JUNO pair contract @ ratio of 0.5 JUNO per HALO"
  );
  const liqResult = await sendTransaction(juno, apTeam, haloTokenContract, {
      increase_allowance: {
        amount: halo_liquidity,
        spender: haloJunoPairContract,
      },
    });
  await sendTransaction(juno, apTeam, haloJunoPairContract, {
      provide_liquidity: {
        assets: [
          {
            info: {
              token: {
                contract_addr: haloTokenContract,
              },
            },
            amount: loop_liquidity,
          },
          {
            info: {
              native_token: {
                denom: "ujuno",
              },
            },
            amount: native_liquidity,
          },
        ],
      },
    },
    [coin(native_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));

  // Farming contract

  // FLP token contract for LOOP/JUNO
  process.stdout.write("Instantiating FLP Token contract for LOOP/JUNO");
  const flpTokenResult1 = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "FLP Token for LOOP/JUNO",
    symbol: "vFLP",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_loop_supply,
      },
      {
        address: apTeam2,
        amount: initial_loop_supply,
      },
      {
        address: apTeam3,
        amount: initial_loop_supply,
      },
    ],
  });
  const flpTokenContract1 = flpTokenResult1.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${flpTokenContract1}`);

  // FLP token contract for HALO/JUNO
  process.stdout.write("Instantiating FLP Token contract for HALO/JUNO");
  const flpTokenResult2 = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "FLP Token for HALO/JUNO",
    symbol: "vFLP",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_loop_supply,
      },
      {
        address: apTeam2,
        amount: initial_loop_supply,
      },
      {
        address: apTeam3,
        amount: initial_loop_supply,
      },
    ],
  });
  const flpTokenContract2 = flpTokenResult2.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${flpTokenContract2}`);

  process.stdout.write("Instantiating the Loopswap Farming contract");
  const farmingResult = await instantiateContract(juno, apTeam, apTeam, farmingCodeId, {
    lp_tokens: [loopJunoPairLP, haloJunoPairLP],
    flp_tokens: [flpTokenContract1, flpTokenContract2],
    loop_token_address: loopTokenContract,
  });

  const farmingContract = farmingResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${farmingContract}`);

  process.stdout.write("Provide the FLP tokens & LOOP tokens for the tests");
  const _f1 = await sendTransaction(juno, apTeam, flpTokenContract1, {
    transfer: {
      recipient: farmingContract,
      amount: "1000000000",
    }
  });

  const _f2 = await sendTransaction(juno, apTeam, flpTokenContract2, {
    transfer: {
      recipient: farmingContract,
      amount: "1000000000",
    }
  });

  const _l = await sendTransaction(juno, apTeam, loopTokenContract, {
    transfer: {
      recipient: farmingContract,
      amount: "1000000000",
    }
  });
  console.log(chalk.green(" Done!"));
}
