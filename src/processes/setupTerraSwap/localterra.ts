/* eslint-disable @typescript-eslint/no-explicit-any */
import * as path from "path";
import chalk from "chalk";
import { LocalTerra, Wallet, MsgExecuteContract } from "@terra-money/terra.js";
import { instantiateContract, sendTransaction, storeCode } from "../../utils/helpers";

// Deploy HALO Token and HALO/UST pair contracts to the LocalTerra
export async function setupTerraSwap(
  terra: LocalTerra,
  apTeam: Wallet,
  ): Promise<void> {
  process.stdout.write("Uploading TerraSwap factory Wasm");
  const factoryCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../artifacts/terraswap_factory.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${factoryCodeId}`);

  process.stdout.write("Uploading TerraSwap pair Wasm");
  const pairCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../artifacts/terraswap_pair.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${pairCodeId}`);

  process.stdout.write("Uploading TerraSwap token Wasm");
  const tokenCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../artifacts/terraswap_token.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${tokenCodeId}`);

  process.stdout.write("Uploading TerraSwap router Wasm");
  const routerCodeId = await storeCode(
    terra,
    apTeam,
    path.resolve(__dirname, "../artifacts/terraswap_router.wasm"));
  console.log(chalk.green(" Done!"), `${chalk.blue("codeId")}=${routerCodeId}`);

  // Factory contract
  process.stdout.write("Instantiating Factory contract");
  const factoryResult = await instantiateContract(terra, apTeam, apTeam, factoryCodeId, {
    pair_code_id: pairCodeId,
    token_code_id: tokenCodeId
  });
  const factoryContract = factoryResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${factoryContract}`);

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(terra, apTeam, apTeam, tokenCodeId, {
    name: "Angel Protocol",
    symbol: "HALO",
    decimals: 6,
    initial_balances: [
      {
        address: apTeam.key.accAddress,
        amount: "1000000000000000"
      }
    ]
  });
  const tokenContract = tokenResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${tokenContract}`);

  // Pair contract
  process.stdout.write("Creating Pair contract from Token Factory");
  const pairResult = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, factoryContract, {
      "create_pair": {
        "asset_infos": [
          {
            "token": {
              "contract_addr": tokenContract,
            }
          },
          {
            "native_token": {
              "denom": "uusd"
            }
          }
        ]
      }
    })
  ]);

  const pairContract = pairResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);
}
