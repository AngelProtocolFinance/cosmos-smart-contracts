/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import fs from "fs";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import {
  sendTransaction,
  sendApplicationViaCw3Proposal,
  CreateMsgCharityEndowment,
  CreateMsgNormalEndowment,
  getWalletAddress,
  clientSetup,
} from "../../utils/helpers/juno";

let client: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeamAddr: string;
let accounts: string;
let cw3ReviewTeam: string;

// setup charity endowments
export async function setupCharityEndowments(
  networkInfo: any,
  endowmentData: CreateMsgCharityEndowment[],
  apTeamWallet: DirectSecp256k1HdWallet,
  cw3ReviewTeam: string,
  accountsContract: string,
  charity_cw3_threshold_abs_perc: string,
  charity_cw3_max_voting_period: number
): Promise<void> {
  apTeam = apTeamWallet;
  accounts = accountsContract;

  let prom = Promise.resolve();
  endowmentData.forEach((item) => {
    prom = prom.then(async () => {
      console.log(`Building new charity endowment for owner: ${item.owner}`);
      const endow_id = await sendApplicationViaCw3Proposal(
        networkInfo,
        apTeam,
        cw3ReviewTeam,
        accounts,
        item.ref_id,
        item.meta,
        {
          owner: item.owner,
          maturity_time: undefined,
          name: item.name,
          categories: item.categories,
          tier: item.tier,
          logo: item.logo,
          image: item.image,
          endow_type: "charity",
          cw4_members: [{ addr: item.owner, weight: 1 }],
          kyc_donors_only: item.kyc_donors_only,
          cw3_threshold: {
            absolute_percentage: { percentage: charity_cw3_threshold_abs_perc },
          },
          cw3_max_voting_period: charity_cw3_max_voting_period,
          beneficiaries_allowlist: [],
          contributors_allowlist: [],
          earnings_fee: undefined,
          withdraw_fee: undefined,
          deposit_fee: undefined,
          aum_fee: undefined,
          dao: undefined,
          proposal_link: undefined,
          settings_controller: undefined,
          parent: undefined,
          split_to_liquid: {
            max: "1.0",
            min: "0.0",
            default: "0.5",
          },
          ignore_user_splits: false,
        },
        [apTeam]
      );
      console.log(chalk.green(`> Endowment ID: ${endow_id} - Done!`));
    });
  });

  await prom;
}

// setup normal endowments
export async function setupNormalEndowments(
  networkInfo: any,
  endowmentData: CreateMsgNormalEndowment[],
  apTeamWallet: DirectSecp256k1HdWallet,
  accountsContract: string
): Promise<void> {
  apTeam = apTeamWallet;
  accounts = accountsContract;
  const sender_addr = await getWalletAddress(apTeamWallet);
  const sender_client = await clientSetup(apTeamWallet, networkInfo);
  let prom = Promise.resolve();
  endowmentData.forEach((item) => {
    prom = prom.then(async () => {
      console.log(`Building new normalized endowment for owner: ${item.owner}`);
      const res = await sendTransaction(sender_client, sender_addr, accounts, {
        create_endowment: {
          owner: item.owner,
          maturity_time: undefined,
          name: item.name,
          categories: item.categories,
          tier: item.tier,
          logo: item.logo,
          image: item.image,
          endow_type: "normal",
          cw4_members: item.cw4_members,
          kyc_donors_only: item.kyc_donors_only,
          cw3_threshold: {
            absolute_percentage: { percentage: item.cw3_threshold },
          },
          cw3_max_voting_period: item.cw3_max_voting_period,
          beneficiaries_allowlist: item.beneficiaries_allowlist,
          contributors_allowlist: item.contributors_allowlist,
          earnings_fee: item.earnings_fee ? item.earnings_fee : undefined,
          withdraw_fee: item.withdraw_fee ? item.withdraw_fee : undefined,
          deposit_fee: item.deposit_fee ? item.deposit_fee : undefined,
          aum_fee: item.aum_fee ? item.aum_fee : undefined,
          dao: item.dao ? item.dao : undefined,
          proposal_link: item.proposal_link ? item.proposal_link : undefined,
          endowment_controller: item.endowment_controller
            ? item.endowment_controller
            : undefined,
          parent: item.parent ? item.parent : undefined,
          split_to_liquid: item.split_to_liquid,
          ignore_user_splits: item.ignore_user_splits,
          referral_id: item.referral_id ? item.referral_id : undefined,
        },
      });
      const endow_id = await parseInt(
        res.logs[0].events
          .find((event) => {
            return event.type == "wasm";
          })
          ?.attributes.find((attribute) => {
            return attribute.key == "endow_id";
          })?.value as string
      );
      console.log(chalk.green(`> Endowment ID: ${endow_id} - Done!`));
    });
  });

  await prom;
}
