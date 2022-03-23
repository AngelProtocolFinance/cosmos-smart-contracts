import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
  testSingleDonationAmountToManyEndowments,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  // testUpdateAngelAllianceMembers,
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
  testAddMemberToC4Group,
  testAddGuardiansToEndowment,
  testGuardiansChangeEndowmentOwner,
} from "./core/multisig";
import {
  testAngelTeamCanTriggerVaultsHarvest,
  testClosingEndpoint,
  testMigrateAllAccounts,
  testUpdatingRegistrarConfigs,
  testQueryRegistrarApprovedVaultList,
  testQueryRegistrarApprovedVaultRateList,
  testQueryRegistrarConfig,
  testQueryRegistrarEndowmentList,
  testQueryRegistrarEndowmentDetails,
  testQueryRegistrarVault,
  testQueryRegistrarVaultList,
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
} from "./lbp/pair";
import { testQueryRouterConfig } from "./lbp/router";
import {
  testTransferTokenBalance,
  testQueryTokenInfo,
  testQueryTokenMinter,
  testPairWithdrawLiquidity,
  testQueryTokenBalance,
} from "./lbp/token";

export async function testExecute(
  terra: LCDClient,
  apTeam: Wallet,
  registrar: string,
  indexFund: string,
  anchorVault: string,
  endowmentContract: string,
  cw4GrpApTeam: string,
  cw4GrpOwners: string,
  cw3ApTeam: string,
  cw3GuardianAngels: string,
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
  lbpLpTokenContract: string
): Promise<void> {
  console.log(chalk.yellow("\nStep 3. Running Tests"));
  // await testSingleDonationAmountToManyEndowments(
  //   terra,
  //   apTeam,
  //   [
  //     "terra1q4sjzkztrpfujqu5vzquhvhvqy872d0drcfuq4", // Legaler Aid
  //   ],
  //   "1000000000"
  // );
  // await testRejectUnapprovedDonations(
  //   terra,
  //   apTeam,
  //   "terra16jm9vflz8ltw9yrrnarcuwt623ampadhhhyxke", // AP Endowment
  //   "000000"
  // );
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateFundMembers(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   19,
  //   ["terra15ej9284yj6v4rm07prxmcrmlhz70p20aup06zh"],
  //   []
  // );
  // await testCreateIndexFund(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   19,
  //   "MVP Rotation #12",
  //   "Fund collection for MVP",
  //   true,
  //   [
  //     "terra1hccjcxm0vdz8d2n9y8lnrpx4ka4elt4gwfm522", // Threshold
  //   ]
  // );
  // await testUpdateAngelAllianceMembers(
  //   terra,
  //   apTeam,
  //   indexFund,
  //   ["terra1gmxefcqt8sfckw0w44tpkuaz0p27eddq76elzx"],
  //   []
  // );
  // await testRemoveIndexFund(terra, apTeam, indexFund, 5);
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 1, [], ["",""]);
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 2, ["",""], []);
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarEndowmentDetails(terra, registrar, endowmentContract1);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault);
  // await testQueryAccountsBalance(terra, endowmentContract);
  // await testQueryVaultConfig(terra, anchorVault);
  // await testQueryAccountsConfig(terra, endowmentContract);
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund, 18);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);

  // HALO gov Tests
  // await testGovUpdateConfig(
  //   terra,
  //   apTeam,
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
  // await testGovResetClaims(terra, apTeam, haloGov, [apTeam.key.accAddress]);
  // await testQueryGovConfig(terra, haloGov);
  // await testQueryGovState(terra, haloGov);
  // await testQueryGovClaims(terra, haloGov, apTeam.key.accAddress);
  // await testQueryGovStaker(terra, haloGov, apTeam.key.accAddress);
  // await testQueryGovPoll(terra, haloGov, 1);
  // await testQueryGovPolls(terra, haloGov, undefined, undefined, undefined);
  // await testQueryGovVoters(terra, haloGov, 1, undefined, undefined);

  // Test query for HALO collector
  // await testCollectorUpdateConfig(
  //   terra,
  //   apTeam,
  //   haloCollector,
  //   "1.0",
  //   undefined,
  //   "terra16hdjuvghcumu6prg22cdjl96ptuay6r0hc6yns"
  // );

  // Test Loop Pair
  // await testPairProvideLiquidity(
  //   terra,
  //   apTeam,
  //   terraswapToken,
  //   "terra1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp", // LOOP PAIR
  //   "13334400000000", //HALO
  //   "1000000000000" //UST
  // );

  // await testPairWithdrawLiquidity(
  //   terra,
  //   apTeam,
  //   lbpPairContract,
  //   lbpLpTokenContract,
  //   "10198039027185"
  // );

  // Test query for LBP Token
  // await testQueryTokenBalance(terra, terraswapToken, apTeam.key.accAddress);

  // await testSendTokenBalance(terra, terraswapToken, apTeam);

  // await testCollectorSweep(terra, apTeam, haloCollector);
}
