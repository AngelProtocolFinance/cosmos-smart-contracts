// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-5",
    walletPrefix: "juno",
    nativeToken: "ujunox",
    gasPrice: "0.025ujunox",
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
    registrar: "juno13ufhg4xjdzylk9mhayc8khmgg25tl2vs42pzala9x53vxa8ppjkschfr0x",
    accounts: "juno1prqanslytzwtrext3qpfy4p83ld7yw04ga06n6yufg53fukf4x0q0udwj2",
    indexFund: "juno107zpvrdyww48d0fylez3lxjf87qwwh8r5nphcdzlwnepnm2kga5q36quta",
    cw4GrpApTeam: "juno13mk4dzwc5qdz7fxcrnkyj448lvap06rp7aw5h34xkcudrm98yv2sz4fysa",
    cw3ApTeam: "juno1pgedpd8m0g76ckxd6fduwpnm6x4g6fzsg0xj4u3xdchvjdxuzckqdhjv9a",
    cw4GrpReviewTeam: "juno1h94wjgxv32zsg64f34retxudwd4nppslwm4glvu2jld9vrqh7k6srzjhcj",
    cw3ReviewTeam: "juno13mlk69qjx2cm8upx3d04h9dxh78mzhfksxrrnuyjk2l5s5wknl8skvkjhp",
    swapRouter: "",
    vaultLocked1: "juno1lem7xy2gam9lm46jax8wzpdyjx4clcqd554zr7mfjrzu2xglh90s8rakew",
    vaultLiquid1: "juno17x84z3vk8k88rqmkladfqaesg2dha8m593kcupszkehdh0u478eqy4kmxr",
    vaultLocked2: "juno1phxdtakp9wdn8w99v7fvddywmhshn6mka68qjexgwhcde97ezyzqwylwl5",
    vaultLiquid2: "juno10vcys0a934r3xnsy6ueprntzv3ymfeumxzy9a9qk9p82qu7fzjxs6jqhrf",
    endowId1: 1,
    endowId2: 2,
    endowId3: 3,
  },

  // JunoSwap contracts
  junoswap: {
    junoswap_code: 0,
    junoswap_token_code: 83,
    junoswap_stake_code: 0,
    halo_token_contract: "",
    halo_juno_pool_contract: "",
    halo_juno_pool_lp_token: "",
    halo_juno_pool_lp_staking_addr: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 JUNO due to faucet limitations
  },


  // Loopswap contracts
  loopswap: {
    loopswap_token_code: 0,
    loopswap_pair_code: 0,

    loopswap_factory: "",
    loopswap_farming: "",

    loop_token_contract: "",
    loop_juno_pair_contract: "",
    loop_juno_pair_lp_token: "",
    initial_loop_supply: "1000000000000000",
    lj_pair_loop_liquidity: "200000000",
    lj_pair_juno_liquidity: "100000000",

    malo_token_contract: "",
    malo_juno_pair_contract: "",
    malo_juno_pair_lp_token: "",
    initial_malo_supply: "1000000000000000",
    mj_pair_malo_liquidity: "20000000",
    mj_pair_juno_liquidity: "10000000",

    kalo_token_contract: "",
    kalo_juno_pair_contract: "",
    kalo_juno_pair_lp_token: "",
    initial_kalo_supply: "1000000000000000",
    kj_pair_kalo_liquidity: "20000000",
    kj_pair_juno_liquidity: "10000000",

    malo_kalo_pair_contract: "",
    malo_kalo_pair_lp_token: "",
    mk_pair_malo_liquidity: "10000000",
    mk_pair_kalo_liquidity: "10000000",
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
    walletPrefix: "juno",
    nativeToken: "ujuno",
    gasPrice: "0.025ujuno",
  },
  mnemonicKeys: {
    apTeam: "",
    apTreasury: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", // Temp use AP Team Admin until we can use the CW3 of the AP Endowment
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "juno17emcut72n6ycmf54qd0l4mzsefqxnqdhqxzlczxstlkkatdlst5qf9s3qr",
    accounts: "juno1e0w8892n60v0juuugvwptj8f6v3ad56ydr3cgxstmpkggjrqzfhsaqh38c",
    indexFund: "juno1yrahlxavwr7juyrty580d24mgvmhknn6h3sgepjtkyg7udvj2l2sujdlqn",
    cw4GrpApTeam: "juno15g9u395kprfhxxzfqhfw56rvwfhjzg8k6mjq82u3yg7fxkhprv8stsu8mm",
    cw3ApTeam: "juno1sae4p8crnac0h9m27psn205d6k586f7cnm4eshws623v05g95teqvj2s8q",
    cw4GrpReviewTeam: "juno1a22f8dxevu3er7vs4lkrca9n8rgf8uvgjd8s2p5eq787vmczq59syuplqx",
    cw3ReviewTeam: "juno1vp2q50smgzw64xm2j2ksntej34pnnedaz4qkwdh8zah9kjcaas6s8g92t8",
    vaults: [],
  },
  members: [
    { addr: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", weight: 1 },
  ],

  // JunoSwap contracts
  junoswap: {
    junoswap_code: 0,
    junoswap_token_code: 83,
    junoswap_stake_code: 0,
    halo_token_contract: "",
    halo_juno_pool_contract: "",
    halo_juno_pool_lp_token: "",
    halo_juno_pool_lp_staking_addr: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "2000000",
    native_liquidity: "1000000", // reduced to 1 JUNO due to faucet limitations
    usdc_juno_pool: "juno1ctsmp54v79x7ea970zejlyws50cj9pkrmw49x46085fn80znjmpqz2n642",
    usdc_juno_pool_staking: "juno1cuu9qxjqukh9drptk2y50r5tvepes7cy55hffh7quvvawk95lxlq6rzzj0",
  },

  // Loopswap contracts
  loopswap: {
    loopswap_token_code: 0,
    loopswap_pair_code: 0,

    loopswap_factory: "",
    loopswap_farming: "",

    loop_token_contract: "",
    loop_juno_pair_contract: "",
    loop_juno_pair_lp_token: "",
    initial_loop_supply: "1000000000000000",
    loop_liquidity: "200000000",
    juno_liquidity: "100000000",

    halo_token_contract: "",
    halo_juno_pair_contract: "",
    halo_juno_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "200000000",
    native_liquidity: "100000000",
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
