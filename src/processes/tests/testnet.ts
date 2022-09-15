import chalk from "chalk";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { datetimeStringToUTC, clientSetup } from "../../utils/helpers";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testUpateAccountsOwner,
  testSendDonationToEndowment,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsProfile,
  testQueryAccountsState,
  testQueryAccountsTransactions,
  testCharityCanHarvestWithdrawFee,
  testCharityCanHarvestAUMFee,
  testEndowmentCanWithdraw,
  testApproveInactiveEndowment,
  testUpdateEndowmentStatus,
  testCreateEndowment,
  testQueryAccountsEndowmentList,
  testEndowmentVaultsRedeem,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAllianceMembersList,
  testUpdatingIndexFundConfigs,
  testCreateIndexFund,
  testRemoveIndexFund,
  testQueryIndexFundActiveFundDetails,
  testQueryIndexFundActiveFundDonations,
  testQueryIndexFundConfig,
  testQueryIndexFundDeposit,
  testQueryIndexFundFundDetails,
  testQueryIndexFundFundsList,
  testQueryIndexFundState,
  testQueryIndexFundTcaList,
  testIndexFundUpdateOwner,
  testIndexFundRemoveMember,
  testIndexFundUpdateAllianceMember,
  testIndexFundUpateRegistrar,
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testAddMemberToC4Group,
  testProposalApprovingEndowment,
  testCw3CastVote,
  testCw3ExecutePoll,
  testQueryMultisigGroupWeight,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryProposal,
  testQueryListProposals,
  testQueryGroupMembersList,
} from "./core/multisig";
import {
  testUpdatingRegistrarConfigs,
  testQueryRegistrarVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarVault,
  testUpdateEndowmentEntry,
  testRegistrarUpdateOwner,
  testUpdateEndowTypeFees,
  testUpdateNetworkConnections,
} from "./core/registrar";
import { testQueryVaultConfig, testVaultHarvest, testVaultReinvestToLocked } from "./core/vaults";
import {
  testAirdropClaim,
  testAirdropRegisterNewMerkleRoot,
  testAirdropUpdateConfig,
  testQueryAirdropMerkleRoot,
  testQueryAirdropConfig,
  testQueryAirdropIsClaimed,
  testQueryAirdropLatestStage,
} from "./halo/airdrop";
import {
  testCollectorUpdateConfig,
  testCollectorSweep,
  testQueryCollectorConfig,
  testQueryCollectorPair,
} from "./halo/collector";
import {
  testCommunityUpdateConfig,
  testCommunitySpend,
  testQueryCommunityConfig,
} from "./halo/community";
import {
  testDistributorUpdateConfig,
  testDistributorAdd,
  testDistributorRemove,
  testDistributorSpend,
  testQueryDistributorConfig,
} from "./halo/distributor";
import {
  testGovCastVote,
  testGovEndPoll,
  testGovExecutePoll,
  testGovRegisterContracts,
  testGovSnapshotPoll,
  testGovUpdateConfig,
  testGovStakeVotingTokens,
  testGovWithdrawVotingTokens,
  testGovClaimVotingTokens,
  testGovExecutePollForRegistrarSettings,
  testQueryGovClaims,
  testQueryGovConfig,
  testQueryGovPoll,
  testQueryGovPolls,
  testQueryGovStaker,
  testQueryGovState,
  testQueryGovVoters,
  VoteOption,
  testGovHodlerUpdateConfig,
  testGovResetClaims,
  testTransferStake,
} from "./halo/gov";
import {
  testStakingUnbond,
  testStakingWithdraw,
  testQueryStakingConfig,
  testQueryStakingStakerInfo,
  testQueryStakingState,
} from "./halo/staking";
import {
  testVestingUpdateConfig,
  testVestingRegisterVestingAccounts,
  testAddSchedulesToVestingAccount,
  testUserClaimsVestedTokens,
  testQueryVestingConfig,
  testQueryVestingAccount,
  testQueryVestingAccounts,
} from "./halo/vesting";
import {
  testInstantiateSubDao,
  testInstantiateSubDaoToken,
  testInstantiateDonationMatchContract,
} from "./core/subdao";

export async function testExecute(
  config: any, // environment config object 
  apTeam: DirectSecp256k1HdWallet,
  apTeam2: DirectSecp256k1HdWallet,
  apTeam3: DirectSecp256k1HdWallet,
  charity1: DirectSecp256k1HdWallet,
  charity2: DirectSecp256k1HdWallet,
  charity3: DirectSecp256k1HdWallet,
  pleb: DirectSecp256k1HdWallet,
  tca: DirectSecp256k1HdWallet,
  apTeamAddr: string,
  apTeam2Addr: string,
  apTeam3Addr: string,
  apTreasuryAddr: string,
  charity1Addr: string,
  charity2Addr: string,
  charity3Addr: string,
  plebAddr: string,
  tcaAddr: string,
  registrar: string,
  indexFund: string,
  vaultLocked1: string,
  vaultLiquid1: string,
  vaultLocked2: string,
  vaultLiquid2: string,
  accounts: string,
  donationMatching: string,
  endowId1: number,
  endowId2: number,
  endowId3: number,
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  cw4GrpReviewTeam: string,
  cw3ReviewTeam: string,
  loopswapFactory: string,
  loopswapFarming: string,
  loopswapLoopJunoPair: string,
  loopswapLoopToken: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
): Promise<void> {
  console.log(chalk.yellow("\nStep 2. Setting up signing clients for all possible actors"));
  const networkUrl = config.networkInfo.url;
  const actors = {
    apTeam: { client: await clientSetup(apTeam, networkUrl), wallet: apTeam, addr: apTeamAddr },
    apTeam2: { client: await clientSetup(apTeam2, networkUrl), wallet: apTeam2, addr: apTeam2Addr },
    apTeam3: { client: await clientSetup(apTeam3, networkUrl), wallet: apTeam3, addr: apTeam3Addr },
    charity1: { client: await clientSetup(charity1, networkUrl), wallet: charity1, addr: charity1Addr },
    charity2: { client: await clientSetup(charity2, networkUrl), wallet: charity2, addr: charity2Addr },
    charity3: { client: await clientSetup(charity3, networkUrl), wallet: charity3, addr: charity3Addr },
    pleb: { client: await clientSetup(pleb, networkUrl), wallet: pleb, addr: plebAddr },
    tca: { client: await clientSetup(tca, networkUrl), wallet: tca, addr: charity3Addr },
  };
  console.log(chalk.green(" Done!"));

  console.log(chalk.yellow("\nStep 3. Running Tests"));

  // Test execute

  // SUBDAO TESTS
  // await testInstantiateSubDao(actors.apTeam.client, actors.apTeam.addr, 165, registrar);
  // await testInstantiateSubDaoToken(actors.apTeam.client, actors.apTeam.addr, 166, registrar);
  // await testInstantiateDonationMatchContract(actors.apTeam.client, actors.apTeam.addr, 167, registrar);


  // Multisig test
  // await testAddMemberToC4Group(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, cw4GrpApTeam, actors.apTeam2.addr);
  // await testUpdateCw3Config(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, 50, 25000);
  // await testProposalApprovingEndowment(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, 1);
  // await testCw3CastVote(actors.apTeam2.client, actors.apTeam2.addr, cw3ApTeam, 7, `yes`);
  // await testCw3ExecutePoll(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, 3);
  // await testQueryMultisigVoters(actors.apTeam.client, cw3ApTeam);
  // await testQueryMultisigThreshold(actors.apTeam.client, cw3ApTeam);
  // await testQueryGroupMembersList(actors.apTeam.client, cw4GrpApTeam);


  // INDEXFUND test
  // await testUpdatingIndexFundConfigs(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund);
  // await testUpdateAllianceMembersList(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   actors.apTeam2.addr, // address #1
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/juno1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   }, // member #1`
  //   "add" // action
  // );
  // await testCreateIndexFund(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   "Test Index Fund Name",
  //   "Test Index Fund desc",
  //   false,
  //   []
  // );
  // await testUpdateFundMembers(actors.apTeam.client, actors.apTeam.addr, indexFund, 2, [], []);
  // await testDonorSendsToIndexFund(actors.pleb.client, actors.pleb.addr, indexFund, 1, "0", "1000000");
  // await testTcaMemberSendsToIndexFund(actors.tca.client, actors.tca.addr, indexFund);
  // await testRemoveIndexFund(actors.apTeam.client, actors.apTeam.addr, indexFund, 1);
  // await testIndexFundUpdateOwner(actors.apTeam.client, actors.apTeam.addr, indexFund, actors.apTeam.addr);
  // await testIndexFundUpateRegistrar(actors.apTeam.client, actors.apTeam.addr, indexFund, registrar); //SHOULDFIXCONTRACT
  // await testIndexFundRemoveMember(actors.apTeam.client, actors.apTeam.addr, indexFund, 1);
  // await testIndexFundUpdateAllianceMember(actors.apTeam.client, actors.apTeam.addr, indexFund, charity3Addr, { name: "Charity3", logo: undefined, website: undefined });


  // REGISTRAR test
  // await testUpdatingRegistrarConfigs(actors.apTeam.client, actors.apTeam.addr, registrar, {
  //   cw3_code: 102,
  //   cw4_code: 104,
  //   accounts_code_id: 102,
  // });

  // await testRegistrarUpdateOwner(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   registrar,
  //   apTeamAddr,
  // );

  // await testUpdateEndowTypeFees(actors.apTeam.client, actors.apTeam.addr, registrar, {
  //   endowtype_charity: "0.05",
  //   endowtype_normal: "0.05",
  // });
  // await testUpdateNetworkConnections(actors.apTeam.client, actors.apTeam.addr, registrar, {
  //   network_info: {
  //     name: "terra-juno channel",
  //     chain_id: "phoenix-1",
  //     ibc_channel: "terra-juno-chann-1",
  //     gas_limit: undefined,
  //   },
  //   action: "add",
  // });

  // ACCOUNTS test
  // await testUpateAccountsOwner(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   apTeamAddr,
  // );

  // await testCreateEndowment(networkUrl, actors.charity1.wallet, cw3ReviewTeam, accounts, {
  //   owner: actors.charity1.addr,
  //   withdraw_before_maturity: false,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   profile: {
  //     name: "Test-Suite Endowment",
  //     overview: "Endowment created from the test-suite integration test",
  //     categories: { sdgs: [2], general: [] },
  //     tier: 3,
  //     logo: "test logo",
  //     image: "test image",
  //     url: undefined,
  //     registration_number: undefined,
  //     country_city_origin: undefined,
  //     contact_email: undefined,
  //     social_media_urls: {
  //       facebook: undefined,
  //       twitter: undefined,
  //       linkedin: undefined,
  //     },
  //     number_of_employees: undefined,
  //     average_annual_budget: undefined,
  //     annual_revenue: undefined,
  //     charity_navigator_rating: undefined,
  //     endow_type: "Normal",
  //   },
  //   kyc_donors_only: false,
  //   whitelisted_beneficiaries: [charity1Addr],
  //   whitelisted_contributors: [],
  //   dao: {
  //     quorum: "0.2",
  //     threshold: "0.5",
  //     voting_period: 1000000,
  //     timelock_period: 1000000,
  //     expiration_period: 1000000,
  //     proposal_deposit: "1000000",
  //     snapshot_period: 1000,
  //     token: {
  //       bonding_curve: {
  //         curve_type: {
  //           square_root: {
  //             slope: "19307000",
  //             power: "428571429",
  //             scale: 9,
  //           }
  //         },
  //         name: "AP Endowment DAO Token",
  //         symbol: "APEDT",
  //         decimals: 6,
  //         reserve_decimals: 6,
  //         reserve_denom: "ujunox",
  //         unbonding_period: 1,
  //       }
  //     }
  //   },
  //   earnings_fee: undefined,
  //   deposit_fee: undefined,
  //   withdraw_fee: undefined,
  //   aum_fee: undefined,
  //   settings_controller: undefined,
  //   parent: false,
  // });
  // await testCharityCanUpdateStrategies(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `locked`,
  //   [{ vault: vaultLocked1, percentage: "0.5"}]
  // );
  // await testCharityCanUpdateStrategies(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `liquid`,
  //   [{ vault: vaultLiquid1, percentage: "0.5"}]
  // );

  // await testSendDonationToEndowment(actors.apTeam.client, actors.apTeam.addr, accounts, endowId1, "1000");
  // await testEndowmentVaultsRedeem(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   `locked`,
  //   [[vaultLocked1, "500000"]],  // Vec<(vault, amount)>
  // );
  // await testVaultHarvest(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   vaultLocked1,
  // );
  // await testVaultReinvestToLocked(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   "1000000",
  //   vaultLiquid1,
  // );

  // await testEndowmentCanWithdraw(
  //   actors.charity1.client,
  //   actors.charity1.addr,
  //   accounts,
  //   endowId1,
  //   vaultLocked1,
  //   "500000",
  //   actors.charity1.addr,
  // );
  // await testApproveInactiveEndowment(actors.apTeam.client, actors.apTeam.addr, cw3ReviewTeam, accounts, 1);
  // await testUpdateEndowmentStatus(actors.apTeam.client, actors.apTeam.addr, accounts, { endowment_id: 1, status: 1, benficiary: undefined });
  // await testRejectUnapprovedDonations(actors.pleb.client, actors.pleb.addr, endowmentContract1, "10000000"); // possible query registrar error


  // Test query
  await testQueryRegistrarConfig(actors.apTeam.client, registrar);
  await testQueryRegistrarVaultList(actors.apTeam.client, registrar);
  await testQueryRegistrarVaultList(actors.apTeam.client, registrar);
  await testQueryRegistrarVault(actors.apTeam.client, registrar, vaultLocked1);

  await testQueryVaultConfig(actors.apTeam.client, vaultLocked1);

  await testQueryAccountsBalance(actors.apTeam.client, accounts, endowId1);
  await testQueryAccountsConfig(actors.apTeam.client, accounts);
  await testQueryAccountsEndowment(actors.apTeam.client, accounts, endowId1);
  await testQueryAccountsProfile(actors.apTeam.client, accounts, endowId1);
  await testQueryAccountsState(actors.apTeam.client, accounts, endowId1);
  await testQueryAccountsEndowmentList(actors.apTeam.client, accounts);

  await testQueryIndexFundConfig(actors.apTeam.client, indexFund);
  await testQueryIndexFundState(actors.apTeam.client, indexFund);
  await testQueryIndexFundTcaList(actors.apTeam.client, indexFund);
  await testQueryIndexFundFundsList(actors.apTeam.client, indexFund, undefined, undefined);
  await testQueryIndexFundFundDetails(actors.apTeam.client, indexFund, 1);
  await testQueryIndexFundActiveFundDetails(actors.apTeam.client, indexFund);
  await testQueryIndexFundActiveFundDonations(actors.apTeam.client, indexFund);
  await testQueryIndexFundDeposit(actors.apTeam.client, indexFund);

  // Test query for HALO airdrop
  // await testAirdropUpdateConfig(actors.apTeam.client, actors.apTeam.addr, apTeam2, pleb, haloAirdrop);
  // await testAirdropRegisterNewMerkleRoot(actors.apTeam.client, actors.apTeam.addr, haloAirdrop);
  // await testAirdropClaim(actors.apTeam.client, actors.apTeam.addr, haloAirdrop);
  // await testQueryAirdropConfig(actors.apTeam.client, haloAirdrop);
  // await testQueryAirdropMerkleRoot(actors.apTeam.client, haloAirdrop, 1);
  // await testQueryAirdropIsClaimed(actors.apTeam.client, haloAirdrop, 1, "juno1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8");
  // await testQueryAirdropLatestStage(actors.apTeam.client, haloAirdrop);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloCollector,
  //   "1.0",
  //   haloGov,
  //   undefined
  // );
  // await testCollectorSweep(actors.apTeam.client, actors.apTeam.addr, haloCollector);
  // await testQueryCollectorConfig(actors.apTeam.client, haloCollector);
  // await testQueryCollectorPair(actors.apTeam.client, haloCollector);

  // Test query for HALO community
  // await testCommunityUpdateConfig(actors.apTeam.client, actors.apTeam.addr, pleb, haloGov, haloCommunity, "1000000", undefined);
  // await testCommunitySpend(actors.apTeam.client, actors.apTeam.addr, haloGov, haloCommunity, "addr000", "1000000");
  // await testQueryCommunityConfig(actors.apTeam.client, haloCommunity);

  // Test query for HALO distributor
  // await testDistributorUpdateConfig(actors.apTeam.client, actors.apTeam.addr, haloDistributor, "1000000", haloGov);
  // await testDistributorSpend(actors.apTeam.client, actors.apTeam.addr, haloDistributor, "addr000", "1000000");
  // await testDistributorAdd(actors.apTeam.client, actors.apTeam.addr, haloGov, haloDistributor, apTeam2Addr);
  // await testDistributorRemove(actors.apTeam.client, actors.apTeam.addr, haloGov, haloDistributor, apTeam2Addr);
  // await testQueryDistributorConfig(actors.apTeam.client, haloDistributor);

  // Tests for HALO vesting
  // await testVestingUpdateConfig(actors.apTeam.client, actors.apTeam.addr, haloVesting, undefined, undefined, undefined);
  // await testVestingRegisterVestingAccounts(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloVesting,
  //   [
  //     {address: apTeam3Addr, schedules: [[100, 101, "100"], [100, 110, "100"], [100, 200, "100"]]},
  //     {address: apTeam2Addr, schedules: [[100, 110, "100"]]},
  //   ]
  // );
  // let new_schedules = [[1000, 2000, "100"]];
  // await testVestingUpdateVestingAccount(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloVesting,
  //   apTeam3Addr,
  //   new_schedules,
  // );
  // await testUserClaimsVestedTokens(actors.apTeam3.client, actors.apTeam3.addr, haloVesting);
  // await testQueryVestingConfig(actors.apTeam.client, haloVesting);
  // await testQueryVestingAccount(actors.apTeam.client, haloVesting, "addr0");
  // await testQueryVestingAccounts(actors.apTeam.client, haloVesting, undefined, undefined);

  // await testGovUpdateConfig(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloGov,
  //   undefined,
  //   15, // quorum
  //   50, // threshold
  //   100800, // voting_period
  //   undefined,
  //   "5000000000", // deposit
  //   100800, // snapshot period
  //   undefined, // unbonding period
  //   undefined // gov_hodler
  // );
  // await testGovExecutePoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovEndPoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovSnapshotPoll(actors.apTeam.client, actors.apTeam.addr, haloGov, 1);
  // await testGovStakeVotingTokens(actors.apTeam.client, actors.apTeam.addr, junoswapToken, haloGov, "20000000000");
  // await testGovStakeVotingTokens(actors.apTeam2.client, actors.apTeam2.addr, junoswapToken, haloGov, "10000000000");
  // await testGovStakeVotingTokens(actors.apTeam3.client, actors.apTeam3.addr, junoswapToken, haloGov, "5000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam.client, actors.apTeam.addr, haloGov, "1000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam2.client, actors.apTeam2.addr, haloGov, "10000000000");
  // await testGovWithdrawVotingTokens(actors.apTeam3.client, actors.apTeam3.addr, haloGov, "10000000000");
  // await testGovClaimVotingTokens(actors.apTeam.client, actors.apTeam.addr, haloGov);
  // await testGovCastVote(actors.apTeam.client, actors.apTeam.addr, haloGov, 1, VoteOption.YES, "1");
  // await testGovRegisterContracts(actors.apTeam.client, actors.apTeam.addr, haloGov, junoswapToken);
  // await testGovExecutePollForRegistrarSettings(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   haloGov,
  //   junoswapToken,
  //   "1000000",
  //   100,
  //   "0.5",
  //   "0.1"
  // );
  // await testGovResetClaims(actors.apTeam.client, actors.apTeam.addr, haloGov, [
  //   apTeamAddr,
  //   apTeam2Addr,
  //   apTeam3Addr,
  // ]);
  // await testQueryGovConfig(actors.apTeam.client, haloGov);
  // await testQueryGovState(actors.apTeam.client, haloGov);
  // await testQueryGovClaims(actors.apTeam.client, haloGov, apTeamAddr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeamAddr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeam2Addr);
  // await testQueryGovStaker(actors.apTeam.client, haloGov, apTeam3Addr);
  // await testQueryGovPoll(actors.apTeam.client, haloGov, 1);
  // await testQueryGovPolls(actors.apTeam.client, haloGov, undefined, undefined, undefined);

  // await testQueryGovVoters(actors.apTeam.client, haloGov, 1, undefined, undefined);

  // Test query for HALO staking
  // await testStakingUnbond(actors.apTeam.client, actors.apTeam.addr, haloStaking, "100");
  // await testStakingWithdraw(actors.apTeam.client, actors.apTeam.addr, haloStaking);
  // await testQueryStakingConfig(actors.apTeam.client, haloStaking);
  // await testQueryStakingStakerInfo(actors.apTeam.client, haloStaking, "addr000", undefined);
  // await testQueryStakingState(actors.apTeam.client, haloStaking);
}
