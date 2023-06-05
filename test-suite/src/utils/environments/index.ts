import * as LocalNet from "./localjuno";
import * as TestNet from "./testnet";
import * as MainNet from "./mainnet";
import * as LocalTerra from "./localterra";
import * as IBC from "./ibc";

export const TestsUniverse = {
  localibc: {
    tests: IBC.startSetupIBC,
    setup: IBC.startSetupIBC,
  },
  localterra: {
    tests: LocalTerra.startTestsAstroportVault,
    setup: {
      astroport: LocalTerra.startSetupAstroport,
      astrovaults: LocalTerra.startSetupAstroportVaults,
    },
  },
  localjuno: {
    tests: LocalNet.startTests,
    setup: {
      core: LocalNet.startSetupCore,
      endowments: LocalNet.startSetupEndowments,
      giftcards: LocalNet.startSetupGiftcards,
      loopswap: LocalNet.startSetupLoopSwap,
      mockvaults: LocalNet.startSetupMockVaults,
      loopvaults: LocalNet.startSetupLoopVaults,
      // "halo": LocalNet.startSetupHalo,
    },
    migrate: {
      core: LocalNet.startMigrateCore,
      // "halo": LocalNet.startMigrateHalo,
    },
  },
  testnet: {
    tests: TestNet.startTests,
    setup: {
      core: TestNet.startSetupCore,
      endowments: TestNet.startSetupEndowments,
      mockvaults: TestNet.startSetupMockVaults,
      loopvaults: TestNet.startSetupLoopVaults,
      giftcards: TestNet.startSetupGiftcards,
      // "halo": t TestNet.startSetupHalo,
      // "junoswap": t TestNet.startSetupJunoSwap,
    },
    migrate: {
      core: TestNet.startMigrateCore,
      // "halo": TestNet.startMigrateHalo,
    },
  },
  mainnet: {
    tests: MainNet.startTests,
    setup: {
      core: MainNet.startSetupCore,
      endowments: MainNet.startSetupEndowments,
      giftcards: MainNet.startSetupGiftcards,
      // "halo": MainNet.startSetupHalo,
      // "junoswap": MainNet.startSetupJunoSwap,
    },
    migrate: {
      core: MainNet.startMigrateCore,
      // "halo": MainNet.startMigrateHalo,
    },
  },
};
