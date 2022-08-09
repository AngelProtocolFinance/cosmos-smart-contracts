/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { coin } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { instantiateContract, sendTransaction, storeCode } from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";

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
  process.stdout.write("Uploading JunoSwap Wasm");
  const swapCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/wasmswap.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${swapCodeId}`);

  process.stdout.write("Uploading JunoSwap token (cw20_base) Wasm");
  const tokenCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/cw20_base.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading JunoSwap stake Wasm");
  const stakerCodeId = await storeCode(
    juno,
    apTeam,
    `${wasm_path.junoswap}/cw20_stake.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${stakerCodeId}`);


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

  // Swap pool contract
  process.stdout.write("Creating Swap pool contract ");
  const swapPoolResult = await instantiateContract(juno, apTeam, apTeam, swapCodeId, {
    "token1_denom": {"native": "ujuno"},
    "token2_denom": {"cw20": tokenContract },
    "lp_token_code_id": tokenCodeId,
  });

  const swapPoolContract = swapPoolResult.contractAddress as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${swapPoolContract}`);

  // Get the LP Token address of newly created swap pool
  process.stdout.write("Query new Swap pool's LP Token contract");
  const result: any = await juno.queryContractSmart(swapPoolContract, {
    info: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.lp_token_address}`
  );

  // send liquidity to the new Swap pool contract for swaps
  process.stdout.write(
    "Add liquidity to the new Swap pool contract @ ratio of 0.05 JUNO per HALO"
  );
  const liqResult = await sendTransaction(juno, apTeam, tokenContract, {
      increase_allowance: {
        amount: halo_liquidity,
        spender: swapPoolContract,
      },
    });
  await sendTransaction(juno, apTeam, swapPoolContract, {
      add_liquidity: {
        token1_amount: native_liquidity,
        min_liquidity: "0",
        max_token2: halo_liquidity,
        expiration: undefined,
      }
    },
    [coin(native_liquidity, "ujuno")]
  );
  console.log(chalk.green(" Done!"));

  // Staking contract
  process.stdout.write("Creating Staking contract for swap pool LP token ");
  const stakeresult = await instantiateContract(juno, apTeam, apTeam, stakerCodeId, {
     owner: undefined,
     // Manager can update all configs except changing the owner. This will generally be an operations multisig for a DAO.
     manager: undefined,
     token_address: result.lp_token_address,
     unstaking_duration: { time: 60 },
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${stakeresult.contractAddress}`
  );
}
