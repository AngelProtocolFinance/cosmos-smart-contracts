/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import {
  sendTransaction,
  instantiateContract,
} from "../../utils/helpers/juno";

chai.use(chaiAsPromised);
const { expect } = chai;

export async function testInstantiateSubDao(
  juno: SigningCosmWasmClient,
  apTeam: string,
  wasm_code: number,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Instantiating a stand-alone SubDao contract");
  const subdao = await instantiateContract(juno, apTeam, apTeam, wasm_code, {
    quorum: "0.2",
    threshold: "0.5",
    voting_period: 1000000,
    timelock_period: 1000000,
    expiration_period: 1000000,
    proposal_deposit: "1000000",
    snapshot_period: 1000,
    endow_type: "Charity",
    endow_owner: "",
    donation_match: undefined,
    registrar_contract: registrar,
    token: {
      bonding_curve: {
        curve_type: {
          square_root: {
            slope: "19307000",
            power: "428571429",
            scale: 9,
          },
        },
        name: "AP Endowment DAO Token",
        symbol: "APEDT",
        decimals: 6,
        reserve_decimals: 6,
        reserve_denom: "ujunox",
        unbonding_period: 1,
      },
    },
  });
  console.log(`Done! Created SubDao: ${subdao.contractAddress}`);
}

export async function testInstantiateSubDaoToken(
  juno: SigningCosmWasmClient,
  apTeam: string,
  wasm_code: number,
  registrar: string
): Promise<void> {
  process.stdout.write(
    "Test - Instantiating a stand-alone SubDao Token contract"
  );
  const subdao = await instantiateContract(juno, apTeam, apTeam, wasm_code, {
    curve_type: {
      square_root: {
        slope: "19307000",
        power: "428571429",
        scale: 9,
      },
    },
    name: "AP Endowment DAO Token",
    symbol: "APEDT",
    decimals: 6,
    reserve_decimals: 6,
    reserve_denom: "ujunox",
    unbonding_period: 1,
  });
  console.log(`Done! Created SubDao Token: ${subdao.contractAddress}`);
}

export async function testInstantiateDonationMatchContract(
  juno: SigningCosmWasmClient,
  apTeam: string,
  wasm_code: number,
  registrar: string
): Promise<void> {
  process.stdout.write("Test - Instantiating a donation matching contract");
  const donationMatch = await instantiateContract(
    juno,
    apTeam,
    apTeam,
    wasm_code,
    {
      registrar_contract: registrar,
      reserve_token: apTeam, // FAKE! Need to fix.
      lp_pair: apTeam, // FAKE! Need to fix.
    }
  );
  console.log(
    `Done! Created Donation Match Contract: ${donationMatch.contractAddress}`
  );
}
