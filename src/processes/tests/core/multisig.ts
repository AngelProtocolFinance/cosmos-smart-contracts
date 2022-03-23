/* eslint-disable @typescript-eslint/no-explicit-any */
import chalk from "chalk";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { LCDClient, LocalTerra, MsgExecuteContract, Wallet } from "@terra-money/terra.js";
import { sendTransaction, toEncodedBinary } from "../../../utils/helpers";

chai.use(chaiAsPromised);
const { expect } = chai;

//----------------------------------------------------------------------------------------
// TEST: Add a new AP Team Member to the C4 AP Team Group
//
// SCENARIO:
// New AP Team Wallet needs to be added to the C4 Group. Done via a new proposal
// by an existing group member, approved with YES votes, and executed by any wallet.
//
//----------------------------------------------------------------------------------------

export async function testAddApTeamMemberToC4Group(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw3ApTeam: string,
  cw4GrpApTeam: string,
  new_member: string
): Promise<void> {
  process.stdout.write("Test - Propose adding a new member to AP Team C4 Group");

  // proposal to add new member
  const proposal = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, cw3ApTeam, {
      propose: {
        title: "New CW4 member",
        description: "New member for the CW4 AP Team Group. They are legit, I swear!",
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: cw4GrpApTeam,
                funds: [],
                msg: toEncodedBinary({
                  update_members: {
                    add: [{ addr: new_member, weight: 1 }],
                    remove: [],
                  },
                }),
              },
            },
          },
        ],
      },
    }),
  ]);
  const proposal_id = proposal.logs[0].events
    .find((event) => {
      return event.type == "wasm";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "proposal_id";
    })?.value as string;

  console.log(chalk.green(" Passed!"));
}

export async function testUpdateCw3Config(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  cw3: string,
  threshold: number,
  max_voting_period: number
): Promise<void> {
  process.stdout.write("Test - Endowment Member Proposes changing the CW3 configs");

  const proposal = await sendTransaction(terra, apTeam, [
    new MsgExecuteContract(apTeam.key.accAddress, cw3, {
      propose: {
        title: "Update CW3 Configurations",
        description: "Changing the max voting period to 48 hours",
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: cw3,
                funds: [],
                msg: toEncodedBinary({
                  update_config: {
                    threshold: { absolute_percentage: { percentage: threshold } },
                    max_voting_period: { height: max_voting_period },
                  },
                }),
              },
            },
          },
        ],
      },
    }),
  ]);
  console.log(chalk.green(" Passed!"));
}

export async function testAddGuardiansToEndowment(
  terra: LocalTerra | LCDClient,
  apTeam3: Wallet,
  charity1: Wallet,
  charity2: Wallet,
  charity3: Wallet,
  pleb: Wallet,
  cw3GuardianAngels: string,
  endowmentContract1: string
): Promise<void> {
  process.stdout.write(
    "Test - Endowment Owner Proposes adding 3 Guardians to their Endowment"
  );

  // proposal to add new Guardians
  const proposal = await sendTransaction(terra, charity1, [
    new MsgExecuteContract(charity1.key.accAddress, cw3GuardianAngels, {
      propose_guardian_change: {
        endowment_addr: endowmentContract1,
        add: [charity3.key.accAddress, apTeam3.key.accAddress, charity2.key.accAddress],
        remove: [],
      },
    }),
  ]);

  const proposal_id = parseInt(
    proposal.logs[0].events
      .find((event) => {
        return event.type == "wasm";
      })
      ?.attributes.find((attribute) => {
        return attribute.key == "proposal_id";
      })?.value as string
  );

  console.log(chalk.green(" Passed!"));
}

// // execute the proposal (anyone can do this for passed proposals)
// await sendTransaction(terra, pleb, [
//   new MsgExecuteContract(pleb.key.accAddress, cw3GuardianAngels, {
//     execute: { proposal_id: proposal_id },
//   }),
// ]);

export async function testGuardiansChangeEndowmentOwner(
  terra: LocalTerra | LCDClient,
  charity2: Wallet,
  charity3: Wallet,
  pleb: Wallet,
  endowmentContract1: string,
  cw3GuardianAngels: string
): Promise<void> {
  process.stdout.write(
    "Test - Endowment Owner loses wallet! :( Guardians Propose, vote and execute a change to new wallet"
  );

  // proposal to add new Guardians
  const proposal = await sendTransaction(terra, charity2, [
    new MsgExecuteContract(charity2.key.accAddress, cw3GuardianAngels, {
      propose_owner_change: {
        endowment_addr: endowmentContract1,
        new_owner_addr: pleb.key.accAddress,
      },
    }),
  ]);

  const proposal_id = parseInt(
    proposal.logs[0].events
      .find((event) => {
        return event.type == "wasm";
      })
      ?.attributes.find((attribute) => {
        return attribute.key == "proposal_id";
      })?.value as string
  );

  // Guardians vote on the open proposal until threshold reached
  await sendTransaction(terra, charity3, [
    new MsgExecuteContract(charity3.key.accAddress, cw3GuardianAngels, {
      vote_guardian: {
        proposal_id: proposal_id,
        vote: "yes",
      },
    }),
    // execute the proposal (anyone can do this for passed proposals)
    new MsgExecuteContract(charity3.key.accAddress, cw3GuardianAngels, {
      execute: { proposal_id: proposal_id },
    }),
  ]);

  console.log(chalk.green(" Passed!"));
}

export async function testQueryMultisigVoters(
  terra: LocalTerra | LCDClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig voters list");
  const result: any = await terra.wasm.contractQuery(multisig, {
    list_voters: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryMultisigThreshold(
  terra: LocalTerra | LCDClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig threshold");
  const result: any = await terra.wasm.contractQuery(multisig, {
    threshold: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}

export async function testQueryGroupMembersList(
  terra: LocalTerra | LCDClient,
  multisig: string
): Promise<void> {
  process.stdout.write("Test - Query a multisig group members list");
  const result: any = await terra.wasm.contractQuery(multisig, {
    list_members: {},
  });

  console.log(result);
  console.log(chalk.green(" Passed!"));
}
