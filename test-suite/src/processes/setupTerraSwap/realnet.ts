/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import { LCDClient, Wallet } from "@terra-money/terra.js";
import { instantiateContract, toEncodedBinary } from "../../utils/helpers";

// Deploy HALO Token and HALO/UST pair contracts to the TestNet/MainNet
export async function setupTerraSwap(
  terra: LCDClient,
  apTeam: Wallet,
  accAddress: string,
  token_code_id: number,
  pair_code_id: number,
  factory_code_id: number,
  factory_contract: string,
  ): Promise<void> {

  // HALO token contract
  process.stdout.write("Instantiating HALO Token contract");
  const tokenResult = await instantiateContract(terra, apTeam, apTeam, token_code_id, {
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
 
  const asset_infos = {
    "create_pair": {
      "asset_infos": [
        {
          "token": {
            "contract_addr": tokenContract
          }
        },
        {
          "native_token": {
            "denom": "uusd"
          }
        }
      ]
    }
  }
  const assetBinary = toEncodedBinary(asset_infos);
  // Pair contract
  process.stdout.write("Instantiating Pair contract");
  const pairResult = await instantiateContract(terra, apTeam, apTeam, pair_code_id, {
    token_code_id: token_code_id,
    asset_infos: [
      { token: { contract_addr: "terra1lxlqekpkwjhacljgcfjm7vxul50wqas6yvuc27" }},
      { native_token: { denom: "uusd" }}
    ],
    init_hook: {
      msg: assetBinary,
      contract_addr: factory_contract
    }
  });
  const pairContract = pairResult.logs[0].events.find((event) => {
    return event.type == "instantiate_contract";
  })?.attributes.find((attribute) => {
    return attribute.key == "contract_address";
  })?.value as string;
  console.log(chalk.green(" Done!"), `${chalk.blue("contractAddress")}=${pairContract}`);
}