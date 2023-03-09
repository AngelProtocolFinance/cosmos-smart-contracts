// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://juno-testnet-rpc.polkachu.com",
    chainId: "uni-6",
    walletPrefix: "juno",
    nativeToken: "ujunox",
    gasPrice: "0.025ujunox",
    axelarGateway:
      "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5",
    axelarIbcChannel: "channel-7",
  },

  mnemonicKeys: {
    apTeam:
      "pact fancy rough prison twenty dismiss mushroom rival page ship quantum deer rookie system cargo",
    apTeam2:
      "knee verify salmon erosion brand ten term three cake help certain bus phrase biology cruel",
    apTeam3:
      "move example spice hint gym liberty weasel drink midnight snow forest vital accident glove dignity",
    apTreasury:
      "truck vacuum tunnel buzz wealth save come short fit kite poverty fork blade venue sword",
    charity1:
      "eager warrior prison into alarm motion annual giggle project silver fabric hover garlic satisfy beach",
    charity2:
      "add buzz humor jump float rotate test rural jazz cave armor pattern update casino undo",
    charity3:
      "lobster worry angry spoil milk cash asthma unhappy number cave object fortune lens uniform simple",
    pleb: "announce reflect cinnamon regular address copper below funny lens draft gain wage inhale gold loyal",
    tca: "camp mom loud claim grass kick tail into cake wear mistake box grunt stand runway",

    junoIbcClient:
      "soda tomato draft between amazing grab suit verb help pony elegant oxygen trial cactus coffee",
  },

  wallets: {
    keeper: "juno1kwzx0d3t4m504xj8hluzcvvkyv9qqa7s529642", // AWS KEEPER WALLET (Donatoooor mngmnt wallet for now)
  },
  
  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar:
      "juno16gyc3r5uxhgqw2mjtxv2795l8fwqn00m9j9fj25cyxk8dls2z5ls82n6vg",
    cw4GrpApTeam:
      "juno1qckn4pkxgpt04azwufygcxm5sx90gvwsgq6d8mc34ugw45tdf3vs6renxh",
    cw3ApTeam:
      "juno1d3ls7cdjaj5tfgsc367ql843wrcztc2g7n7lupejujgeqnrvw5fqh9hm5s",
    settingsController:
      "juno1ed9a2268ksgv7fufutnx3sxcyqxnmn839efyrkvnqpuyc0es5x9qs24j5z",
    accounts: "juno1d4v5h5yr920xjjwcuret5n3meee5l04c58fpjhghdgml8kp57spqvmg4lx",
    indexFund:
      "juno1zkxltk53aa8p4djzc76nuxtyf2pnygjy64ky0e48rhf7c3r0gk7sv0y7r3",
    cw4GrpReviewTeam:
      "juno1ge3qfdl2p3fe99ysffcshh9d8fe4tp94d5n0tmqf8fjs69gxvuqqd5kx32",
    cw3ReviewTeam:
      "juno198p9uuzwwurjfa5fwfg0exm65h2ldn363kuh6mtkxmxuxwe0yy8q7y4jws",
    swapRouter:
      "juno15mgqc7cep5v9l3w4kkaednyvw5ve79njkzs4t72wxeq26rurgt6s483kpk",
    donationMatching: "",
    giftcards: "",
    vaultLocked1: "",
    vaultLiquid1: "",
    vaultLocked2: "",
    vaultLiquid2: "",
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
    // loop_liquidity: "200000000",
    // juno_liquidity: "100000000",

    // halo_token_contract: "",
    // halo_juno_pair_contract: "",
    // halo_juno_pair_lp_token: "",
    // initial_halo_supply: "1000000000000000",
    // halo_liquidity: "200000000",
    // native_liquidity: "100000000",
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
    url: "https://juno-rpc.angelprotocol.io",
    chainId: "juno-1",
    walletPrefix: "juno",
    nativeToken: "ujuno",
    gasPrice: "0.025ujuno",
    axelarGateway: "???",
    axelarIbcChannel: "???",
  },

  wallets: {
    apTreasury:
      "juno1fz8jx4qhkgdrm5vm3s20n670mf872xsp2c0a6cl0yuncquzmj4jss2cfgj", // this is the CW3 of the AP Endowment
    keeper: "juno1kwzx0d3t4m504xj8hluzcvvkyv9qqa7s529642", // AWS KEEPER WALLET (Donatoooor mngmnt wallet for now)
  },
  
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar:
      "juno17emcut72n6ycmf54qd0l4mzsefqxnqdhqxzlczxstlkkatdlst5qf9s3qr",
    accounts: "juno1e0w8892n60v0juuugvwptj8f6v3ad56ydr3cgxstmpkggjrqzfhsaqh38c",
    indexFund:
      "juno1yrahlxavwr7juyrty580d24mgvmhknn6h3sgepjtkyg7udvj2l2sujdlqn",
    cw4GrpApTeam:
      "juno15g9u395kprfhxxzfqhfw56rvwfhjzg8k6mjq82u3yg7fxkhprv8stsu8mm",
    cw3ApTeam:
      "juno1sae4p8crnac0h9m27psn205d6k586f7cnm4eshws623v05g95teqvj2s8q",
    cw4GrpReviewTeam:
      "juno1a22f8dxevu3er7vs4lkrca9n8rgf8uvgjd8s2p5eq787vmczq59syuplqx",
    cw3ReviewTeam:
      "juno1vp2q50smgzw64xm2j2ksntej34pnnedaz4qkwdh8zah9kjcaas6s8g92t8",
    swapRouter: "",
    donationMatching: "",
    giftcards:
      "juno17pghl3qreyqnjlq6hun5ymshl0dkfeelcy738dkgk602lzmgcvaq2e4xav",
    settingsController: "",
    vaults: [],
  },
  members: [{ addr: "juno1q6n47p729sla2jekc45rlmrvwchkj4gc39a296", weight: 1 }],

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
    usdc_juno_pool:
      "juno1ctsmp54v79x7ea970zejlyws50cj9pkrmw49x46085fn80znjmpqz2n642",
    usdc_juno_pool_staking:
      "juno1cuu9qxjqukh9drptk2y50r5tvepes7cy55hffh7quvvawk95lxlq6rzzj0",
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
