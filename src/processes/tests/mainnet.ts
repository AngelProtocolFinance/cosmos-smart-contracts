import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js";
import chalk from "chalk";
import {
  testBeneficiaryCanWithdrawFromLiquid,
  testCharityCanUpdateStrategies,
  testRejectUnapprovedDonations,
  testQueryAccountsBalance,
  testQueryAccountsConfig,
  testQueryAccountsEndowment,
} from "./core/accounts";
import {
  testDonorSendsToIndexFund,
  testTcaMemberSendsToIndexFund,
  testUpdateFundMembers,
  testUpdateAngelAllianceMembers,
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
  testAddApTeamMemberToC4Group,
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
  testVestingUpdateVestingAccount,
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
import {
  testRouterSwapOperations,
  testQueryRouterConfig,
  testQueryRouterSimulateSwapOperations,
} from "./lbp/router";
import {
  testQueryTokenBalance,
  testQueryTokenInfo,
  testQueryTokenMarketingInfo,
  testQueryTokenMinter,
  testPairWithdrawLiquidity,
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
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateFundMembers(terra, apTeam, indexFund, 12, [
  //     "terra1yfemvj4epgx74j8jm0gfl3n2qen2w9q6eyhan8", // Imagine Worldwide
  //   ],
  //   []
  // );
  // await testCreateIndexFund(terra, apTeam, indexFund, 12, "MVP Rotation #6", "Fund collection for MVP", true, [
  //     "terra1vm7g8ah6v95xs4d8q774fhavfrc5f2lzf9fygs", // Alex's Lemonade Stand Foundation
  // ]);
  // await testUpdateAngelAllianceMembers(terra, apTeam, indexFund, [
  //   "terra1zxtczmxtw8mk8xncvr8lcq2qmvk4dz88ek6f79", // community
  //   "terra1janh9rs6pme3tdwhyag2lmsr2xv6wzhcrjz0xx", // community
  //   "terra18n2pc9x6q9str9dz8sqpt7ulz5telutclkzaec", // lunapes
  //   "terra17me29hk8cdd6mm6uf7cf0amsxmzxnszkfe5ph4", // lunabulls
  //   "terra1r59snugfm3gxjas565jf5ehw54junlfpmspjan", // lunabulls
  //   "terra1tz9jtxemq5e9sw048adz32tj62vkzp6f63e26f", // Astronorcs
  //   "terra1pl2cus25j79ukff04fxn9wwemerm2463gnztl6", // west coast
  //   "terra1etwq0q8wwnmq7322kz4v6ff2dcvwfm634vdkqn", // loop finance
  //   "terra157vv7nqra4zpfa58cglen5ekqmekxqw5ss3edq", // loop finance
  //   "terra1feqtlvaru4lszqnpjesgfw8splrg7u27wwwqac", // loop finance
  //   "terra1rzjxj4c6ykemk8csvtjchcqas7mul8s4w6rk8x", // tales of terra
  //   "terra1kf4k0l7hj5tlkuzf67ly43q8d2gcxay3hwa7fr", // hero
  //   "terra1yvg94g6ydgme2kdhy5t4gasgvem2kpk56g4h5e", // Luna Millionares Portrait
  //   "terra14amh70rm5a3wjgkf7trvten3jfqum2svppky3e", // Terra Terrapins
  //   "terra1amrl8f5fqen2m478nuh2z7mz5ce096x4xqae9p", // Woof of Luna
  //   "terra1hxrd8pnqytqpelape3aemprw3a023wryw7p0xn", // ApolloDAO
  // ]);
  // await testRemoveIndexFund(terra, apTeam, indexFund, 5);
  // await testUpdatingIndexFundConfigs(terra, apTeam, indexFund);
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 1, [], ["",""]);
  // await testUpdateFundMembers(terra, apTeam, pleb, indexFund, 2, ["",""], []);
  // Test query
  // await testQueryRegistrarConfig(terra, registrar);
  // await testQueryRegistrarEndowmentList(terra, registrar);
  // await testQueryRegistrarApprovedVaultList(terra, registrar);
  // await testQueryRegistrarApprovedVaultRateList(terra, registrar);
  // await testQueryRegistrarVaultList(terra, registrar);
  // await testQueryRegistrarVault(terra, registrar, anchorVault);
  // await testQueryAccountsBalance(terra, endowmentContract);
  // await testQueryVaultConfig(terra, anchorVault);
  // await testQueryAccountsConfig(terra, endowmentContract);
  // await testQueryAccountsEndowment(terra, endowmentContract);
  // await testQueryIndexFundConfig(terra, indexFund);
  // await testQueryIndexFundState(terra, indexFund);
  // await testQueryIndexFundTcaList(terra, indexFund);
  // await testQueryIndexFundFundsList(terra, indexFund);
  // await testQueryIndexFundFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDetails(terra, indexFund);
  // await testQueryIndexFundActiveFundDonations(terra, indexFund);
  // await testQueryIndexFundDeposit(terra, indexFund);
}
