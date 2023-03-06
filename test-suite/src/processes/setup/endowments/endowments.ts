/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import fs from "fs";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import {
  sendTransaction,
  sendApplicationViaCw3Proposal,
  Endowment,
} from "../../../utils/juno/helpers";

let client: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeamAddr: string;
let accounts: string;
let cw3ReviewTeam: string;

// setup charity endowments
export async function setupEndowments(
  networkInfo: any,
  endowmentData: Endowment[],
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
      console.log(`Building new endowment for owner: ${item.owner}`);
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
          categories: { sdgs: item.un_sdgs, general: [] },
          tier: item.tier,
          logo: item.logo,
          image: item.image,
          url: item.url,
          endow_type: "Charity",
          cw4_members: [{ addr: item.owner, weight: 1 }],
          kyc_donors_only: item.kyc_donors_only,
          cw3_threshold: {
            absolute_percentage: { percentage: charity_cw3_threshold_abs_perc },
          },
          cw3_max_voting_period: charity_cw3_max_voting_period,
          whitelisted_beneficiaries: [],
          whitelisted_contributors: [],
          split_max: "1.0",
          split_min: "0.0",
          split_default: "0.5",
          earnings_fee: undefined,
          withdraw_fee: undefined,
          deposit_fee: undefined,
          aum_fee: undefined,
          dao: undefined, // Option<DaoSetup>,      // SubDAO setup options
          proposal_link: undefined, // Option<u64>, // link back to the proposal that created an Endowment (set @ init)
          settings_controller: undefined, // Option<SettingsController>,
          parent: undefined, // Option<u64>
          split_to_liquid: undefined,
          ignore_user_splits: false,
        },
        [apTeam]
      );
      console.log(chalk.green(`> Endowment ID: ${endow_id} - Done!`));
    });
  });

  await prom;
}
