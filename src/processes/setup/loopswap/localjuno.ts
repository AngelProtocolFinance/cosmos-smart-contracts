/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { coin } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { instantiateContract, sendTransaction, sendTransactionWithFunds, storeCode } from "../../../utils/juno/helpers";
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

  initial_malo_supply: string,
  maloJunoPair_malo_liquidity: string,
  maloJunoPair_juno_liquidity: string,

  initial_kalo_supply: string,
  kaloJunoPair_kalo_liquidity: string,
  kaloJunoPair_juno_liquidity: string,

  maloKaloPair_malo_liquidity: string,
  maloKaloPair_kalo_liquidity: string,
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
  await sendTransactionWithFunds(juno, apTeam, loopJunoPairContract, {
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


  // MALO token contract
  process.stdout.write("Instantiating MALO Token contract");
  const maloTokenResult = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "MALO Token",
    symbol: "MALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_malo_supply,
      },
      {
        address: apTeam2,
        amount: initial_malo_supply,
      },
      {
        address: apTeam3,
        amount: initial_malo_supply,
      },
    ],
  });
  const maloTokenContract = maloTokenResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${maloTokenContract}`);

  // MALO/JUNO contract
  process.stdout.write("Creating MALO/JUNO pair contract ");
  const maloJunoPairResult = await sendTransaction(juno, apTeam, factoryContract, {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: maloTokenContract,
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

  const maloJunoPairContract = maloJunoPairResult.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "pair_contract_addr";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${maloJunoPairContract}`);

  // Get the LP Token address of newly created swap pool
  process.stdout.write("Query new MALO/JUNO pool's LP Token contract");
  const result1: any = await juno.queryContractSmart(maloJunoPairContract, {
    pair: {},
  });
  const maloJunoPairLP = result1.liquidity_token as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${maloJunoPairLP}`
  );

  // send liquidity to the new MALO/JUNO pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new MALO/JUNO pair contract @ ratio of 0.5 JUNO per MALO"
  );
  const liqResult = await sendTransaction(juno, apTeam, maloTokenContract, {
    increase_allowance: {
      amount: maloJunoPair_malo_liquidity,
      spender: maloJunoPairContract,
    },
  });
  await sendTransactionWithFunds(juno, apTeam, maloJunoPairContract, {
    provide_liquidity: {
      assets: [
        {
          info: {
            token: {
              contract_addr: maloTokenContract,
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
          amount: maloJunoPair_juno_liquidity,
        },
      ],
    },
  },
    [coin(maloJunoPair_juno_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));

  // KALO token contract
  process.stdout.write("Instantiating KALO Token contract");
  const kaloTokenResult = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "KALO Token",
    symbol: "KALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam,
        amount: initial_kalo_supply,
      },
      {
        address: apTeam2,
        amount: initial_kalo_supply,
      },
      {
        address: apTeam3,
        amount: initial_kalo_supply,
      },
    ],
  });
  const kaloTokenContract = kaloTokenResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${kaloTokenContract}`);

  // KALO/JUNO contract
  process.stdout.write("Creating KALO/JUNO pair contract ");
  const kaloJunoPairResult = await sendTransaction(juno, apTeam, factoryContract, {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: kaloTokenContract,
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

  const kaloJunoPairContract = kaloJunoPairResult.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "pair_contract_addr";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${kaloJunoPairContract}`);

  // Get the LP Token address of newly created swap pool
  process.stdout.write("Query new KALO/JUNO pool's LP Token contract");
  const result2: any = await juno.queryContractSmart(kaloJunoPairContract, {
    pair: {},
  });
  const kaloJunoPairLP = result2.liquidity_token as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${kaloJunoPairLP}`
  );

  // send liquidity to the new KALO/JUNO pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new KALO/JUNO pair contract @ ratio of 0.5 JUNO per KALO"
  );
  const liqResul = await sendTransaction(juno, apTeam, kaloTokenContract, {
    increase_allowance: {
      amount: kaloJunoPair_kalo_liquidity,
      spender: kaloJunoPairContract,
    },
  });
  await sendTransactionWithFunds(juno, apTeam, kaloJunoPairContract, {
    provide_liquidity: {
      assets: [
        {
          info: {
            token: {
              contract_addr: kaloTokenContract,
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
          amount: kaloJunoPair_juno_liquidity,
        },
      ],
    },
  },
    [coin(kaloJunoPair_juno_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));

  // MALO/KALO contract
  process.stdout.write("Creating MALO/KALO pair contract ");
  const maloKaloPairResult = await sendTransaction(juno, apTeam, factoryContract, {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: maloTokenContract,
          },
        },
        {
          token: {
            contract_addr: kaloTokenContract,
          },
        },
      ],
    },
  });

  const maloKaloPairContract = maloKaloPairResult.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "pair_contract_addr";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${maloKaloPairContract}`);

  // Get the LP Token address of newly created swap pool
  process.stdout.write("Query new MALO/KALO pool's LP Token contract");
  const result3: any = await juno.queryContractSmart(maloKaloPairContract, {
    pair: {},
  });
  const maloKaloPairLP = result3.liquidity_token as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${maloKaloPairLP}`
  );

  // send liquidity to the new MALO/KALO pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new MALO/KALO pair contract @ ratio of 1 MALO per KALO"
  );
  const liqRes = await sendTransaction(juno, apTeam, maloTokenContract, {
    increase_allowance: {
      amount: maloKaloPair_malo_liquidity,
      spender: maloKaloPairContract,
    },
  });
  const liqResu = await sendTransaction(juno, apTeam, kaloTokenContract, {
    increase_allowance: {
      amount: maloKaloPair_kalo_liquidity,
      spender: maloKaloPairContract,
    },
  });
  await sendTransactionWithFunds(juno, apTeam, maloKaloPairContract, {
    provide_liquidity: {
      assets: [
        {
          info: {
            token: {
              contract_addr: maloTokenContract,
            },
          },
          amount: maloKaloPair_malo_liquidity,
        },
        {
          info: {
            token: {
              contract_addr: kaloTokenContract,
            },
          },
          amount: maloKaloPair_kalo_liquidity,
        },
      ],
    },
  },
    []
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

  // FLP token contract for MALO/KALO
  process.stdout.write("Instantiating FLP Token contract for MALO/KALO");
  const flpTokenResult2 = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
    name: "FLP Token for MALO/KALO",
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
    lp_tokens: [loopJunoPairLP, maloKaloPairLP],
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
