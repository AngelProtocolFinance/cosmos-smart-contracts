// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-3",
  },
  mnemonicKeys: {
    apTeam: "pact fancy rough prison twenty dismiss mushroom rival page ship quantum deer rookie system cargo",
    apTeam2: "knee verify salmon erosion brand ten term three cake help certain bus phrase biology cruel",
    apTeam3: "move example spice hint gym liberty weasel drink midnight snow forest vital accident glove dignity",
    apTreasury: "truck vacuum tunnel buzz wealth save come short fit kite poverty fork blade venue sword",
    charity1: "eager warrior prison into alarm motion annual giggle project silver fabric hover garlic satisfy beach",
    charity2: "add buzz humor jump float rotate test rural jazz cave armor pattern update casino undo",
    charity3: "lobster worry angry spoil milk cash asthma unhappy number cave object fortune lens uniform simple",
    pleb: "announce reflect cinnamon regular address copper below funny lens draft gain wage inhale gold loyal",
    tca: "camp mom loud claim grass kick tail into cake wear mistake box grunt stand runway",
  },
  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar: "juno1tc8u3xvll720jlp76svt540laaxtmhx3v0483xtcax8w8phs9h5s2mcy3l",
    indexFund: "juno1zqx658llgl8k607zj8ntg0h0yjselpra50tw6jkwuxeq295gex3qkmwcm9",
    vault1: "",
    vault2: "",
    endowmentContract1: "juno1km3zssn9n7yz5klucca5nl94pxnmq0srvedp4hwnzqzqzs5aynuq80el28",
    endowmentContract2: "juno1gylgx73qfk3jump9zws57y6nm29ruwg5trzx0sgakw9kv257v9sqj3lxg9",
    endowmentContract3: "juno1k898dsn6w676n8ynxvt0dns3z4suc3szeeaughf6hlfjpac0sx4qdw49tx",
    endowmentContract4: "juno133dr6d3q7c873a5u2qte9vg7av6lz4qveu20weguvglumrhy8tjqjhn04x",
    cw4GrpApTeam: "juno1rp7jge4mrj0yfdl4pea68xrcwege6x6ezreaylf0t9exxhxdpa7qrslsfm",
    cw3ApTeam: "juno1ut5gzsatw49vzarh4nvtl0mcgpe6ngw9k5pys2ntr2kkp5rfnq6sdezxcr",
  },

  // JunoSwap contracts
  junoswap: {
    junoswap_token_code: 83,
    junoswap_factory: "",
    halo_token_contract: "",
    halo_luna_pair_contract: "",
    halo_luna_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 JUNO due to faucet limitations
  },

  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    gov_hodler_contract: "",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;

// ---------------------------------------------------------------------------------------------------
// MainNet information
// ---------------------------------------------------------------------------------------------------
export const mainnet = {
  // MainNet phoenix-1
  networkInfo: {
    url: "https://rpc-juno.itastakers.com",
    chainId: "juno-1",
  },
  mnemonicKeys: {
    apTeam: "",
    apTreasury: "",
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "",
    indexFund: "",
    anchorVault: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    endowmentContracts: [],
  },
  members: [
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
    { addr: "", weight: 1 },
  ],

  // JunoSwap contracts
  junoswap: {
    junoswap_token_code: 4,
    junoswap_factory: "",
    halo_token_contract: "",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "",
    native_liquidity: "",
  },

  // LOOP LP contracts
  loop: {
    junoswap_factory: "",
    pair_contract: "",
    lp_token_contract: "",
  },

  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    gov_hodler_contract: "",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;
