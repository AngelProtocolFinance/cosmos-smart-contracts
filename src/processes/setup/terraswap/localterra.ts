/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LocalTerra, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, storeCode } from "../../../utils/helpers";
import { wasm_path } from "../../../config/wasmPaths";

// Deploy HALO Token and HALO/UST pair contracts to the LocalTerra
export async function setupTerraSwap(
  terra: LocalTerra,
  apTeam: Wallet,
  initial_halo_supply: string,
  halo_liquidity: string,
  native_liquidity: string
): Promise<void> {
  process.stdout.write("Uploading TerraSwap factory Wasm");
  const factoryCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.terraswap}/terraswap_factory.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading TerraSwap pair Wasm");
  const pairCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.terraswap}/terraswap_pair.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading TerraSwap token Wasm");
  const tokenCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.terraswap}/terraswap_token.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading TerraSwap router Wasm");
  const routerCodeId = await storeCode(
    terra,
    apTeam,
    `${wasm_path.terraswap}/terraswap_router.wasm`
  );
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // Factory contract
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId,
  });
  const factoryContract = factoryResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${factoryContract}`
  );

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(terra, apTeam, apTeam, tokenCodeId, {
    name: "Angel Protocol",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam.key.accAddress,
        amount: initial_halo_supply,
      },
    ],
  });
  const tokenContract = tokenResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Token Factory");
  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      create_pair: {
        asset_infos: [
          {
            token: {
              contract_addr: tokenContract,
            },
          },
          {
            native_token: {
              denom: "uusd",
            },
          },
        ],
      },
    }),
  ]);

  const pairContract = pairResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate_contract";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "contract_address";
    })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);

  // Get the LP Token address of newly created pair
  process.stdout.write("Query new Pair's LP Token contract");
  const result: any = await terra.wasm.contractQuery(pairContract, {
    pair: {},
  });
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("contractAddress")}=${result.liquidity_token}`
  );

  // send liquidity to the new Pair contract for swaps
  process.stdout.write(
    "Provide liquidity to the new Pair contract @ ratio of 0.05 UST per HALO"
  );
  const liqResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, tokenContract, {
      increase_allowance: {
        amount: halo_liquidity,
        spender: pairContract,
      },
    }),
    new MsgExecuteContract(
      apTeam.key.accAddress,
      pairContract,
      {
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
                  denom: "uusd",
                },
              },
              amount: native_liquidity,
            },
          ],
        },
      },
      {
        uusd: native_liquidity,
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));
}
