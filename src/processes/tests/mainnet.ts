import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import chalk from "chalk";
import {
  testEndowmentCanWithdrawLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testSendDonationToEndowment,
  testUpdateEndowmentStatus,
  testQueryAccountsEndowmentList,
  testCreateNormalEndowment,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
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
  testQueryIndexFundInvolvedAddress,
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testAddMemberToC4Group,
  testProposalApprovingEndowment,
  testCw3CastVote,
  testCw3ExecutePoll,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryGroupMembersList,
} from "./core/multisig";
import {
  testUpdatingRegistrarConfigs,
  testQueryRegistrarVaultList,
  testQueryRegistrarConfig,
  testQueryRegistrarVault,
} from "./core/registrar";
import { testQueryVaultConfig } from "./core/vaults";
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
  testGovWithdrawVotingTokens,
  testGovExecutePollForRegistrarSettings,
  testQueryGovConfig,
  testQueryGovPoll,
  testQueryGovPolls,
  testQueryGovStaker,
  testQueryGovState,
  testQueryGovVoters,
  VoteOption,
  testGovHodlerUpdateConfig,
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
  // testVestingUpdateVestingAccount,
  testQueryVestingConfig,
  testQueryVestingAccount,
  testQueryVestingAccounts,
} from "./halo/vesting";

export async function testExecute(
  juno: SigningCosmWasmClient,
  apTeam: DirectSecp256k1HdWallet,
  apTeamAddr: string,
  registrar: string,
  indexFund: string,
  accounts: string,
  cw4GrpApTeam: string,
  cw3ApTeam: string,
  cw4GrpReviewTeam: string,
  cw3ReviewTeam: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testQueryMultisigVoters(juno, cw3ReviewTeam);

  /* --- ACCOUNTS & ENDOWMENTS --- */
  // await testCreateNormalEndowment(juno, apTeamAddr, accounts, {
  //   owner: apTeamAddr,
  //   withdraw_before_maturity: true,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   profile: {
  //     name: "Test Normal Endowment",
  //     overview: "Test normalized endowment created from the test-suite",
  //     categories: { sdgs: [], general: [] },
  //     tier: 3,
  //     logo: "logo 1",
  //     image: "logo 1",
  //     url: "",
  //     registration_number: undefined,
  //     country_city_origin: undefined,
  //     contact_email: "",
  //     social_media_urls: {
  //       facebook: undefined,
  //       twitter: undefined,
  //       linkedin: undefined,
  //     },
  //     number_of_employees: 1,
  //     average_annual_budget: undefined,
  //     annual_revenue: undefined,
  //     charity_navigator_rating: undefined,
  //     endow_type: "Normal",
  //   },
  //   cw4_members: [{ addr: apTeamAddr, weight: 1 }],
  //   kyc_donors_only: false,
  //   cw3_threshold: { absolute_percentage: { percentage: "0.5" } },
  //   cw3_max_voting_period: 604800,
  // });

  // await testAngelTeamCanTriggerVaultsHarvest(
  //   juno,
  //   apTeamAddr,
  //   charity1,
  //   registrar,
  //   haloCollector,
  //   "0.5"
  // );
  // await testSendDonationToEndowment(
  //   juno,
  //   apTeamAddr,
  //   "juno1d6lkyls54z5rpqw8d4x738etn9zvt3cw35ya0r", // Coalition for Engaged Education
  //   "1000000000"
  // );
  // await testRejectUnapprovedDonations(
  //   juno,
  //   apTeamAddr,
  //   "juno16jm9vflz8ltw9yrrnarcuwt623ampadhhhyxke", // AP Endowment
  //   "000000"
  // );
  // await testUpdatingIndexFundConfigs(juno, apTeamAddr, indexFund);
  // await testUpdateFundMembers(
  //   juno,
  //   apTeamAddr,
  //   indexFund,
  //   23,
  //   [
  //     "juno1uegpp7nuxazgf20medwy4dwdhvkzvdztmrj8jx", // PEPA
  //   ],
  //   []
  // );
  // await testCreateIndexFund(
  //   juno,
  //   apTeamAddr,
  //   indexFund,
  //   "MVP Rotation #14",
  //   "Fund collection for MVP",
  //   true,
  //   [
  //     "juno14hmdpqwr49j7vyeqmjmp9zxsym0fczp66kuz0g", // Mauti Cancer
  //   ]
  // );
  // await testUpdateAngelAllianceMembers(
  //   juno,
  //   apTeamAddr,
  //   indexFund,
  //   ["juno1gmxefcqt8sfckw0w44tpkuaz0p27eddq76elzx"],
  //   []
  // );
  // await testRemoveIndexFund(juno, apTeamAddr, indexFund, 5);
  // await testUpdatingIndexFundConfigs(juno, apTeamAddr, indexFund);
  // await testUpdateFundMembers(juno, apTeamAddr, pleb, indexFund, 1, [], ["",""]);
  // await testUpdateFundMembers(juno, apTeamAddr, pleb, indexFund, 2, ["",""], []);

  // await testUpdateEndowmentsStatus(juno, apTeamAddr, registrar, [
  //   {
  //     address: "juno1vqe93uv8lylkw4fc8m0xr89fv5xean29ftr0q2",
  //     status: 3,
  //     beneficiary: "juno1suxqzxtzztxvakvucc6u4s9833n4u0cyk9pmv8",
  //   },
  // ]);

  // Test query
  // await testQueryRegistrarConfig(juno, registrar);
  // await testQueryRegistrarEndowmentDetails(juno, registrar, endowmentContract1);
  // await testQueryRegistrarVaultList(juno, registrar);
  // await testQueryRegistrarVaultList(juno, registrar);
  // await testQueryRegistrarVault(juno, registrar, anchorVault);
  await testQueryAccountsEndowmentList(juno, accounts);
  // await testQueryAccountsBalance(juno, endowmentContract);
  // await testQueryVaultConfig(juno, anchorVault);
  // await testQueryAccountsConfig(juno, accounts);
  // await testQueryIndexFundConfig(juno, indexFund);
  // await testQueryIndexFundState(juno, indexFund);
  // await testQueryIndexFundTcaList(juno, indexFund);
  // await testQueryIndexFundFundsList(juno, indexFund, 10, 20);
  // await testQueryIndexFundFundDetails(juno, indexFund, 23);
  // await testQueryIndexFundActiveFundDetails(juno, indexFund);
  // await testQueryIndexFundActiveFundDonations(juno, indexFund);
  // await testQueryIndexFundDeposit(juno, indexFund);
  // await testQueryIndexFundInvolvedAddress(
  //   juno,
  //   indexFund,
  //   "juno1vqe93uv8lylkw4fc8m0xr89fv5xean29ftr0q2"
  // );

  // HALO gov Tests
  // await testGovUpdateConfig(
  //   juno,
  //   apTeamAddr,
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
  // await testGovResetClaims(juno, apTeamAddr, haloGov, [apTeamAddr]);
  // await testQueryGovConfig(juno, haloGov);
  // await testQueryGovState(juno, haloGov);
  // await testQueryGovClaims(juno, haloGov, apTeamAddr);
  // await testQueryGovStaker(juno, haloGov, apTeamAddr);
  // await testQueryGovPoll(juno, haloGov, 1);
  // await testQueryGovPolls(juno, haloGov, undefined, undefined, undefined);
  // await testQueryGovVoters(juno, haloGov, 1, undefined, undefined);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   juno,
  //   apTeamAddr,
  //   haloCollector,
  //   "1.0",
  //   undefined,
  //   "juno16hdjuvghcumu6prg22cdjl96ptuay6r0hc6yns"
  // );

  // Test Loop Pair
  // await testPairProvideLiquidity(
  //   juno,
  //   apTeamAddr,
  //   junoswapToken,
  //   "juno1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp", // LOOP PAIR
  //   "13334400000000", //HALO
  //   "1000000000000" //axlUSDC
  // );

  // await testPairWithdrawLiquidity(
  //   juno,
  //   apTeamAddr,
  //   lbpPairContract,
  //   lbpLpTokenContract,
  //   "10198039027185"
  // );

  // Test query for LBP Token
  // await testQueryTokenBalance(juno, junoswapToken, apTeamAddr);

  // await testSendTokenBalance(juno, junoswapToken, apTeamAddr);

  // await testCollectorSweep(juno, apTeamAddr, haloCollector);
}
