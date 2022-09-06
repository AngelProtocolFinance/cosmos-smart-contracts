// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.junomint.com:443/", // "https://rpc.uni.juno.deuslabs.fi",
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
    registrar: "juno1xdcdynleemkjmlwcz44wnl3px8h4ctueec8segt6dm58g4dzw5uq4ne3cz",
    accounts: "juno1eudag2gs6q5fgf0gxga5dz468fuj3kwnd72vys8alnn4x62zefhsqxg330",
    indexFund: "juno17qjnys68aemf9nzncyr7gt8yscquzxm792mc35nklwqyzk7qcvxqwt9l5m",
    cw4GrpApTeam: "juno10vnq2rpvh7ztyuefzh6dre9pzdyvg7ve8ghev3ufgeyu227wt2fq5e07du",
    cw3ApTeam: "juno1mcq9sg7y2ms589w7tj63u2ta3g57gnv554z26tdnnjnldklsju7qmxn6zt",
    cw4GrpReviewTeam: "juno1mlm82v0yascqgsf3sngfvn67w6mqhjw3ysr6eeyndzden6ggsh3qlmne93",
    cw3ReviewTeam: "juno1whhz5ywu6ff7ghhyt76v0dc586mk5ehncmmfxnf6u0hxu6uxfsdq8arm4p",
    vault1: "juno1ppd785xpearupc0p46a9fr7rzhjqczwmtsv2nclgrtwdpug9wfys4zr6u5",
    vault2: "juno1qtrv875q4829aq7dcz4x7awtmxugxq0eqkrysrmq0q3qnmspuywqu4g2ce",
    endowId1: 1,
    endowId2: 2,
    endowId3: 3,
    endowId4: 4,
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
    accounts: "",
    indexFund: "",
    anchorVault: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    cw4GrpReviewTeam: "",
    cw3ReviewTeam: "",
    endowmentIDs: [],
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
