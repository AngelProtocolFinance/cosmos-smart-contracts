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
    registrar: "juno1tlnfqkkg4axjla5xuzdp54lvsw9v37m356lkzng6vsvarmfe68vqm4z8p0",
    indexFund: "juno1627ftf5dg0duuuwy822qtymv8juxk2rcrfkx7d9xzrv8c5gtkcgsh3pp8n",
    vault1: "",
    vault2: "",
    endowmentContract1: "juno1evze5lv5n92xn8qyzsjdl6upelngskklyuhq5jk7guypqazsy88s64m7z0",
    endowmentContract2: "juno1x23yl84tssavlakr7ud0qf86mplhlfdpclxf79ah8jkdjl3p9n4qg3efp0",
    endowmentContract3: "juno1maghj6e3tfc3pt5uk0axeaxsu9uy3w5ezaruqa2h5x6etgp3vwzqqkk73y",
    endowmentContract4: "juno1laz848qkgfguta2gu278sl76609m6v3f2p6tjdy77jtkz78mnllq33vna5",
    cw4GrpApTeam: "juno1g8f5yur65qam4ugl98gtjk7e2ssxjwrtksx9sphzy5t79ycds9gqea8dzv",
    cw3ApTeam: "juno1weuhhjmdlj3h45rcsruyn556wdpnt0gl64qh30r4flhsfsdgf67segplnj",
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
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    endowmentContracts: [],
    anchorVault: "",
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
