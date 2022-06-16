import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import { localterra } from "../../config/localterraConstants";
import { datetimeStringToUTC } from "../../utils/helpers";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testApTeamChangesAccountsEndowmentOwner,
  testChangeManyAccountsEndowmentOwners,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testQueryAccountsProfile,
  testQueryAccountsState,
  testQueryAccountsTransactions,
  testCharityCanHarvestWithdrawFee,
  testCharityCanHarvestAUMFee,
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
} from "./core/indexFunds";
import {
  testUpdateCw3Config,
  testAddMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner,
  testQueryMultisigVoters,
  testQueryMultisigThreshold,
  testQueryGroupMembersList,
} from "./core/multisig";
import {
  testUpdateEndowmentsStatus,
  testCreateEndowmentViaRegistrar,
  testAngelTeamCanTriggerVaultsHarvest,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarEndowmentDetails,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
  testUpdateEndowmentsEntry,
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
  testFactoryUpdateConfig,
  testFactoryCreatePair,
  testFactoryUnregister,
  testQueryFactoryConfig,
  testQueryFactoryPair,
  testQueryFactoryPairs,
} from "./lbp/factory";
import {
  testQueryPairPair,
  testQueryPairPool,
  testQueryPairReverseSimulationNativeToHalo,
  testQueryPairReverseSimulationHaloToNative,
  testQueryPairSimulationNativeToHalo,
  testQueryPairSimulationHaloToNative,
  testPairProvideLiquidity,
  testPairSwapHaloToNative,
  testPairSwapNativeToHalo,
  getPairContractLpToken,
} from "./lbp/pair";
import { testQueryRouterConfig } from "./lbp/router";
import {
  testQueryTokenBalance,
  testQueryTokenInfo,
  testQueryTokenMinter,
  testPairWithdrawLiquidity,
  testTransferTokenBalance,
} from "./lbp/token";

export async function testExecute(
  terra: LocalTerra | LCDClient,
  apTeam: Wallet,
  apTeam2: Wallet,
  apTeam3: Wallet,
  charity1: Wallet,
  charity2: Wallet,
  charity3: Wallet,
  pleb: Wallet,
  tca: Wallet,
  registrar: string,
  indexFund: string,
  anchorVault1: string,
  anchorVault2: string,
  endowmentContract1: string,
  endowmentContract2: string,
  endowmentContract3: string,
  endowmentContract4: string,
  cw4GrpApTeam: string,
  // cw4GrpOwners: string,
  cw3ApTeam: string,
  // cw3GuardianAngels: string,
  terraswapFactory: string,
  terraswapToken: string,
  terraswapPair: string,
  haloAirdrop: string,
  haloCollector: string,
  haloCommunity: string,
  haloDistributor: string,
  haloGov: string,
  haloStaking: string,
  haloVesting: string,
  lbpFactoryContract: string,
  lbpPairContract: string,
  lbpRouterContract: string,
  lbpLpTokenContract: string,
  slippageTolerance: string | undefined
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateAllianceMembersList(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   "terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf", // address #1
  //   {
  //     name: "Testnet Charity #2",
  //     website:
  //       "http://angelprotocol.io/app/charity/terra1w0fn5u7puxafp3g2mehe6xvt4w2x2eennm7tzf",
  //     logo: "https://angelprotocol.io/favicon.ico",
  //   }, // member #1`
  //   // "terra178u9lz89f54njqz6nentst3m9nye2cc7ezssmq", // address #2
  //   // { name: "Testnet Admin", webiste: "http://angelprotocol.io", logo: "" }, // member #2
  //   "add" // action
  // );
  // await testRemoveIndexFund(terra, apTeam, indexFund, 5);
  // await testCreateIndexFund(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   "Test Index Fund Name",
  //   "Test Index Fund desc",
  //   false,
  //   []
  // );
  // await testUpdateFundMembers(terra, apTeam, indexFund, 2, [], []);
  // await testChangeManyAccountsEndowmentOwners(terra, apTeam, [
  //   {
  //     "address": "terra16zj5dw97sk7q3rvakzu76uyfv6zrxkvsln0yjz2wa5s58mq67vhs5wdv7l", // Current one is localterra endow1.
  //     "owner": apTeam, 
  //     "kyc_donors_only": false,
  //   }
  // ]);

  // await testCreateEndowmentViaRegistrar(terra, apTeam, registrar, {
  //   owner: charity1.key.accAddress,
  //   beneficiary: charity1.key.accAddress,
  //   withdraw_before_maturity: false,
  //   maturity_time: undefined,
  //   maturity_height: undefined,
  //   guardians_multisig_addr: undefined,
  //   cw4_members: [{ addr: charity1.key.accAddress, weight: 1 }],
  //   kyc_donors_only: false,
  //   profile: {
  //     name: "Test-Suite Endowment",
  //     overview: "Endowment created from the test-suite integration test",
  //     un_sdg: 2,
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
  //     endow_type: "Charity",
  //   },
  // });
  // Multisig test
  // await testAddMemberToC4Group(terra, apTeam, cw3ApTeam, cw4GrpApTeam, apTeam.key.accAddress);
  // await testUpdateCw3Config(terra, apTeam, cw3ApTeam, 50, 25000);

  // await testQueryMultisigVoters(terra, cw3ApTeam);
  // await testQueryMultisigThreshold(terra, cw3ApTeam);
  // await testQueryGroupMembersList(terra, cw4GrpApTeam);

  // Test execute
  // await testRejectUnapprovedDonations(terra, pleb, endowmentContract1, "10000000"); // possible query registrar error
  // await testDonorSendsToIndexFund(terra, pleb, indexFund, 1, "0.5", "4200000"); // possible query registrar error
  // await testTcaMemberSendsToIndexFund(terra, tca, indexFund); // possible query registrar error
  // await testAngelTeamCanTriggerVaultsHarvest(
  //   terra,
  //   apTeam,
  //   charity1,
  //   registrar,
  //   haloCollector,
  //   "0.5"
  // );  // vault-related
  // await testCharityCanUpdateStrategies(
  //   terra,
  //   charity1,
  //   endowmentContract1,
  //   anchorVault1,
  //   anchorVault2
  // );  // vault-related
}
