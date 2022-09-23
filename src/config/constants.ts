// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  networkInfo: {
    url: "https://rpc.uni.junomint.com:443/", // "https://rpc.uni.juno.deuslabs.fi",
    chainId: "uni-4",
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
    registrar: "juno19vfquz8hz0dp773ct9q39anqghw7lzqpdzafwcqjl0c7tn5mz0vqmvww28",
    accounts: "juno1wj0dak9qwv6jtay0knldyupg7a56m2awpezyh8u407v66la5q9tqt345ng",
    indexFund: "juno16rz2d0n4sjuwlzwgwxn3xuk7zarr2u2css3v5qy6azc9f002tgrqhekful",
    cw4GrpApTeam: "juno1lr0jcep4ckr7fwcteu9jgndj4nycstksajy2fjy3ap4n2usc67ss7tqsep",
    cw3ApTeam: "juno1qur3fztscxwutxpl2xk3gsmy442gqneskj95q8zpmvmvd7zam96sxdq55k",
    cw4GrpReviewTeam: "juno1qf6jqdr633ry7e52hq9kels98vlzxh2lt983w79qafvdzse3dk6s6zecnw",
    cw3ReviewTeam: "juno1wtkvj07sxk8wa2sjsvaneym8ly5nzhvzdwwukelmf4ywklqdy96qlhllvt",
    vaultLocked1: "juno1vay5qjs5aa5jgmm0qq6z9yffwtu4ntz3s883mc02efsjsv2m7t9qm4cfsc",
    vaultLiquid1: "juno17svz6ft6dkk6p828tedcdsuw0erra3agpgu3tpak3q07gz89st7ssp94js",
    vaultLocked2: "juno1s2u3wkw6djexf7ygnqzlesa6ea249lhyl9j3ldkgaj4px5jdveds8xrlpe",
    vaultLiquid2: "juno1ttf9hg76vhw7scv6qnn9xtu2006elj3gnk5x3qudgfv59z7xe9zsmyjxrs",
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
    registrar: "juno16uva5mgmzj78rdwf5hcqv688lyenms6s32zfjectg8zkr4vc3xfq95eueu",
    accounts: "juno1wkfy4u8zj45jqeeyvl7pedzcvaksutug56d0v32p6qw3fmpyw6rq82y648",
    indexFund: "juno1ycpml96cru0ln20zv7qxkc6xuass8lerpk0cfwvgmvyn6zjux55srqkah5",
    cw4GrpApTeam: "juno1ucjmf3nztyq4a6q8tzja9thefq7092s9r4a2yk8267spr2fays2qks9rl4",
    cw3ApTeam: "juno1dftgv4yhy8yqx95c7a3jar9dg5nnq4p2m50nzk6wdlkahd2h4hms3js63a",
    cw4GrpReviewTeam: "juno1p2yg2xy7z39mhnn4r248nfsf9zufa7kxh34g7kh9u7zvjacm4nssx2x6gv",
    cw3ReviewTeam: "juno1et8z5we83ny06j5sra9q9vaxw6e9cp4qjy5agy8emqz8wcmmgheqnq5xdy",
    vaults: [],
  },
  members: [
    { addr: "juno1c2ha303qdjlcpphjjv7s7r8ns2lnqm9ec79lfa", weight: 1 },
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
