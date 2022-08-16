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
    registrar: "juno1ea06p4lms9ku2dca9vxnta28nwl5t0z7pr7e50jh0xhm4ujkp35szz7sae",
    accounts: "juno1mfqqu85m0mvc0ahsrz57c8d4x0x0teyq4jmfq92vuy8fgwv0uepq9uj0kr",
    indexFund: "juno19l9msl003n0zrnl3xsccr4ypvhxyamaqddv0fqh45lf0dwtyplxqef4xp8",
    cw4GrpApTeam: "juno1nmetyh7phu89rp8pl8rxdtyzkcan8tp90899er9jnqjf9pylzm8sypfqhq",
    cw3ApTeam: "juno1ttn32lntds7tk64fcy8ldz30t9frhy3uxg5lzak2ej0jd4md0szqda2nuc",
    vault1: "",
    vault2: "",

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
