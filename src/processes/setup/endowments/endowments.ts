/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import fs from "fs";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { sendTransaction, sendApplicationViaCw3Proposal, Endowment } from "../../../utils/helpers";

let client: SigningCosmWasmClient;
let apTeam: DirectSecp256k1HdWallet;
let apTeamAddr: string;
let accounts: string;
let cw3ReviewTeam: string;
let endowments: Endowment[] = [];
let endowmentIds: any[] = [];

// setup charity endowments
export async function setupEndowments(
  networkUrl: string,
  endowmentData: Endowment[],
  apTeamWallet: DirectSecp256k1HdWallet,
  cw3ReviewTeam: string,
  accountsContract: string,
  charity_cw3_threshold_abs_perc: string,
  charity_cw3_max_voting_period: number,
): Promise<void> {
  networkUrl = networkUrl;
  apTeam = apTeamWallet;
  cw3ReviewTeam = cw3ReviewTeam;
  accounts = accountsContract;

  let prom = Promise.resolve();
  endowmentData.forEach((item) => {
    // eslint-disable-next-line no-async-promise-executor
    prom = prom.then(
      () =>
        new Promise(async (resolve, reject) => {
          try {
            console.log(`Building new endowment for owner: ${item.owner}`);
            let endow_id = await sendApplicationViaCw3Proposal(networkUrl, apTeam, cw3ReviewTeam, accounts, "unknown", {
              owner: item.owner,
              withdraw_before_maturity: false,
              maturity_time: undefined,
              maturity_height: undefined,
              profile: {
                name: item.name,
                overview: item.overview,
                categories: { sdgs: [item.un_sdg], general: [] },
                tier: item.tier,
                logo: item.logo,
                image: item.image,
                url: item.url,
                registration_number: item.registration_number,
                country_of_origin: item.country_of_origin,
                street_address: item.street_address,
                contact_email: item.email,
                social_media_urls: {
                  facebook: item.facebook_page,
                  twitter: item.twitter_handle,
                  linkedin: item.linkedin_page,
                },
                number_of_employees: item.number_of_employees,
                average_annual_budget: item.average_annual_budget,
                annual_revenue: item.annual_revenue,
                charity_navigator_rating: item.charity_navigator_rating,
                endow_type: "Charity",
              },
              cw4_members: [{ addr: item.owner, weight: 1 }],
              kyc_donors_only: false,
              cw3_threshold: { absolute_percentage: { percentage: charity_cw3_threshold_abs_perc } },
              cw3_max_voting_period: charity_cw3_max_voting_period,
            }, [apTeamWallet]);
            console.log(chalk.green(`> Endowment ID: ${endow_id} - Done!`));
            resolve();
          } catch (e) {
            reject(e);
          }
        })
    );
  });

  await prom;
}
