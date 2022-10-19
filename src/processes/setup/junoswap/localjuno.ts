/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { instantiateContract, sendTransaction, storeCode } from "../../../utils/juno/helpers";
import { wasm_path } from "../../../config/wasmPaths";
import { coin } from "@cosmjs/stargate";

// Deploy HALO Token and HALO/JUNO pair contracts to the LocalJuno
export async function setupJunoSwap(
  juno: SigningCosmWasmClient,
  apTeam: string,
  apTeam2: string,
  apTeam3: string,
  initial_halo_supply: string,
  halo_liquidity: string,
  native_liquidity: string
): Promise<void> {
  process.stdout.write("Uploading JunoSwap factory Wasm");
  const factoryCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/junoswap_factory.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading JunoSwap pair Wasm");
  const pairCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/junoswap_pair.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading JunoSwap token Wasm");
  const tokenCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/junoswap_token.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading JunoSwap router Wasm");
  const routerCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/junoswap_router.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // Factory contract
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(juno, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
  });
  const factoryContract = factoryResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${factoryContract}`
  );

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(juno, apTeam, apTeam, tokenCodeId, {
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
  const tokenContract = tokenResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Token Factory");
  const pairResult = await sendTransaction(juno, apTeam, "factoryContract", {
    create_pair: {
      asset_infos: [
        {
          token: {
            contract_addr: tokenContract,
          },
        },
        {
          native_token: {
            // denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
            denom: "ujuno",
          },
        },
      ],
    },
    // }, { "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4": "100" },
  }, [coin(1000, "ujuno")],
  );

  const pairContract = "pairResult.contractAddress" as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  // Get the LP Token address of newly created pair
  process.stdout.write("Query new Pair's LP Token contract");
  const result: any = await juno.queryContractSmart(pairContract, {
    pair: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.liquidity_token}`
  );

  // send liquidity to the new Pair contract for swaps
  process.stdout.write(
    "Provide liquidity to the new Pair contract @ ratio of 0.05 JUNO per HALO"
  );
  const liqResult = await sendTransaction(juno, apTeam, tokenContract, {
    increase_allowance: {
      amount: halo_liquidity,
      spender: pairContract,
    },
  });
  await sendTransaction(juno, apTeam, pairContract, {
    provide_liquidity: {
      assets: [
        {
          info: {
            token: {
              contract_addr: tokenContract,
            },
          },
          amount: halo_liquidity,
        },
        {
          info: {
            native_token: {
              // denom: "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4",
              denom: "ujuno",
            },
          },
          amount: native_liquidity,
        },
      ]
    },
  },
    [coin(native_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));
}