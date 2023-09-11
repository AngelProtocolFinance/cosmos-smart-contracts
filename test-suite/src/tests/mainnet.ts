import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import chalk from "chalk";
import {
  testEndowmentCanWithdrawLiquid,
  testCharityCanWithdrawLocked,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testApTeamChangesEndowmentSettings,
  testSendDonationToEndowment,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsState,
  testApproveInactiveEndowment,
  testUpdateEndowmentStatus,
  testCreateEndowment,
  testEndowmentVaultsRedeem,
  testSendRestitutionFundsToEndowments,
  testCloseEndowment,
  testDistributeEndowmentToBeneficary,
  // buildNewEndowmentCw3sAndChangeOwner,
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
  testQueryIndexFundState,
  testQueryIndexFundInvolvedAddress,
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testUpdateCw3ApplicationsConfig,
  testAddMemberToC4Group,
  testProposalApprovingEndowment,
  testCw3CastVote,
  testCw3ExecutePoll,
  testCreateEndowmentCw3,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryGroupMembersList,
  testQueryProposal,
  testQueryProposalList,
  testQueryMultisigConfig,
  testQueryApplicationsCw3Balances,
} from "./core/multisig";
import {
  testUpdatingRegistrarConfigs,
  testQueryRegistrarConfig,
  testQueryRegistrarStrategy,
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
import {
  testSendDepositToGiftcards,
  testClaimGiftcardsDeposit,
  testSpendGiftcardsBalance,
  testQueryGiftcardsBalance,
  testQueryGiftcardsConfig,
  testQueryGiftcardsDeposit,
} from "./core/accessories";

export async function testExecute(
  juno: SigningCosmWasmClient,
  apTeam: DirectSecp256k1HdWallet,
  apTeamAddr: string,
  registrar: string,
  indexFund: string,
  accounts: string,
  settingsController: string,
  donationMatching: string,
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
  giftcards: string
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testCloseEndowment(juno, apTeamAddr, accounts, 77);
  // await testDistributeEndowmentToBeneficary(juno, apTeamAddr, accounts, 77);

  // await testAddMemberToC4Group(juno, apTeamAddr, cw3ReviewTeam, cw4GrpReviewTeam, "juno1h27pex3z3mm97gwfdhan8cfak8yzvtvprjlcz7");
  // await testQueryMultisigConfig(juno, cw3ReviewTeam);
  // await testUpdateCw3ApplicationsConfig(juno, apTeamAddr, cw3ReviewTeam, "0.3", 50000, "100000000", "0", "180000");
  // await testCw3ExecutePoll(juno, apTeamAddr, cw3ReviewTeam, 178);
  // await testCw3CastApplicationVote(juno, apTeamAddr, cw3ReviewTeam, 44, `yes`);
  // await testQueryProposal(juno, cw3ReviewTeam, 181);
  // await testQueryMultisigVoters(juno, cw3ReviewTeam);
  // await testQueryMultisigThreshold(juno, cw3ReviewTeam);
  // await testQueryApplicationsCw3Balances(juno, cw3ReviewTeam);

  /* --- REGISTRAR contract --- */
  // await testUpdatingRegistrarUpdateOwner(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, registrar, cw3ApTeam);
  // await testUpdatingRegistrarConfigs(juno, apTeamAddr, cw3ApTeam, registrar, {
  //   cw3_code: 1098,
  // });
  // await testUpdatingRegistrarNetworkConnections(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   registrar,
  //   {
  //     name: "Juno mainnet",
  //     chain_id: "juno-1",
  //     ibc_channel: undefined,
  //     ica_address: undefined,
  //     gas_limit: undefined,
  //   },
  //   "post", // action: "post" or "delete"
  // );

  /* --- INDEXFUND contract --- */
  // await testUpdatingIndexFundOwner(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, cw3ApTeam);
  // await testUpdatingIndexFundRegistrar(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, registrar);
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
  //   // "juno178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // address #2
  //   // { name: "Testnet Admin", webiste: "http://angelprotocol.io", logo: "" }, // member #2
  //   "add" // action
  // );
  // await testUpdateAllianceMember(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   actors.apTeam2.addr, // member address
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/juno1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   },
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
  // await testRemoveIndexFund(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   7,
  // );
  // await testIndexFundRemoveMember(actors.apTeam.client, actors.apTeam.addr, cw3ApTeam, indexFund, 1); // INCOMPLETE: "remove_member" entry should be called from "accounts_contract".
  // await testDonorSendsToIndexFund(actors.pleb.client, actors.pleb.addr, indexFund, 1, "0", "1000000");
  // await testTcaMemberSendsToIndexFund(actors.tca.client, actors.tca.addr, indexFund);
  // await testUpdateFundMembers(
  //   actors.apTeam.client,
  //   actors.apTeam.addr,
  //   cw3ApTeam,
  //   indexFund,
  //   2,
  //   [1, 2],
  //   []
  // );

  /* --- ACCOUNTS & ENDOWMENTS --- */
  // const endowments_batch = [];
  // await testSendRestitutionFundsToEndowments(
  //   juno,
  //   apTeamAddr,
  //   accounts,
  //   endowments_batch,
  //   "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034"
  // );
  // await testCreateEndowmentCw3(juno, apTeamAddr, registrar, accounts, 176, "juno1...124");
  // await testApTeamChangesEndowmentSettings(juno, apTeamAddr, cw3ApTeam, accounts, endowments_batch);
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
  //   accounts,
  //   25,
  //   { denom: "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034", amount: "1371000000" }
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

  // await testUpdateEndowmentStatus(juno, apTeamAddr, accounts, {
  //   endowment_id: 1,
  //   status: 3,
  //   beneficiary: { wallet: { address: apTeamAddr } }
  // });
  // await testUpdateEndowmentStatus(juno, apTeamAddr, accounts, {
  //   endowment_id: 159,
  //   status: 3,
  //   beneficiary: { endowment: { id: 166 } }
  // });

  // Test query
  // await testQueryRegistrarConfig(juno, registrar);
  // await testQueryRegistrarEndowmentDetails(juno, registrar, endowmentContract1);
  // await testQueryRegistrarVaultList(juno, registrar);
  // await testQueryRegistrarVaultList(juno, registrar);
  // await testQueryRegistrarVault(juno, registrar, anchorVault);
  await testQueryAccountsConfig(juno, accounts);
  await testQueryAccountsState(juno, accounts, 77);
  await testQueryAccountsEndowment(juno, accounts, 77);
  // await testQueryVaultConfig(juno, anchorVault);
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

  /* --- GIFTCARD contract --- */
  // await testSendDepositToGiftcards(juno, apTeamAddr, giftcards, { denom: "ujunox", amount: "4206900" });
  // await testClaimGiftcardsDeposit(juno, apTeamAddr, giftcards, 1, actors.apTeam2.addr);
  // await testSpendGiftcardsBalance(juno, apTeam2Addr, giftcards, "ujuno", "100000", 22, "0", "1");
  // await testQueryGiftcardsConfig(juno, giftcards);
  // await testQueryGiftcardsDeposit(juno, giftcards, 1);
  // await testQueryGiftcardsBalance(juno, giftcards, "juno1nat09n7vfkgrv3p78vyan203umugmrkxe9mcrz");

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
