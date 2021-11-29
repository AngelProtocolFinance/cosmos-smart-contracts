// ---------------------------------------------------------------------------------------------------
// LocalTerra information
// ---------------------------------------------------------------------------------------------------
export const localterra = {
  // LocalTerra
  networkInfo: {
    url: "http://localhost:1317",
    chainId: "localterra",
  },
  // LocalTerra AP / DANO Treasury wallet (HALO collector contract)
  apTreasury: "",
  // Should be updated contract addresses after deploying wasms in the LocalTerra
  contracts: {
    registrar: "",
    indexFund: "",
    anchorVault1: "",
    anchorVault2: "",
    endowmentContract1: "",
    endowmentContract2: "",
    endowmentContract3: "",
    endowmentContract4: "",
    cw4GrpApTeam: "",
    cw3ApTeam: "",
    cw4GrpOwners: "",
    cw3GuardianAngels: "",
  },
  // TerraSwap balances account
  token_code_id: 0,
  pair_code_id: 0,
  factory_code_id: 0,

  // LBP contracts
  factory_contract: "",
  token_contract: "",
  pair_contract: "",
  router_contract: "",
  
  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    staking_contract: "",
    vesting_contract: "",
  }
}

// ---------------------------------------------------------------------------------------------------
// TestNet information
// ---------------------------------------------------------------------------------------------------
export const testnet = {
  // TestNet bombay-12
  networkInfo: {
    url: "https://bombay-lcd.terra.dev",
    chainId: "bombay-12",
  },
  // TestNet MoneyMarket Contract
  anchorMoneyMarket: "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal",
  // TestNet AP / DANO Treasury wallet (HALO collector contract)
  apTreasury: "terra14d0uufjsmaktaanxtnlm38cullcq449gu35x0q",
  mnemonicKeys: {
    apTeam: "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
    apTeam2: "custom review state crisp modify sell trick replace bone wolf ridge paper later collect topple income owner head turkey estate canyon tone copy inhale",
    apTeam3: "law cause body surround problem join swift shy lumber start immense spray mandate organ pledge butter modify fossil pluck demise link bus rebel misery",
    charity1: "source multiply curtain modify nurse party valid awesome road local focus retreat route agree spot rule false cloud dwarf six relief clay unhappy thank",
    charity2: "stick dumb cabin wish great impact fork save trade crime today seed tortoise base enter topic physical glue maple cliff over myth marble loyal",
    charity3: "write obscure shop lunar fruit attend media abuse spirit lens illegal pluck rally cave stamp gadget burger rigid minute index paper voice eight again",
    pleb: "shoot cry panther mesh blind embrace bottom exchange forest dad polar popular siege idea sure guard disorder toss above tube gaze finish whip column",
    tca: "win height tragic load when music day issue game track promote midnight desert ordinary thunder barely ahead wealth bundle force spray shop cushion mystery",
  },
  // Should be updated contract addresses after deploying wasms in the testnet
  contracts: {
    registrar: "terra15upcsqpg57earvp7mc49kl5e7cppptu2ndmpak",
    indexFund: "terra1typpfzq9ynmvrt6tt459epfqn4gqejhy6lmu7d",
    anchorVault1: "terra1mvtfa3zkayfvczqdrwahpj8wlurucdykm8s2zg",
    anchorVault2: "terra16y7du2keuersslsevvqx32z04wy6juyfwjs3ru",
    endowmentContract1: "terra1grjzys0n9n9h9ytkwjsjv5mdhz7dzurdsmrj4v",
    endowmentContract2: "terra1glqvyurcm6elnw2wl90kwlhtzrd2zc7q00prc9",
    endowmentContract3: "terra1vyw5r7n02epkpk2tm2lzt67fyv28qzalwzgzuu",
    endowmentContract4: "terra1jvtf3ccpkr3vymv98vk9nz7wvwmykgv8yk9l3w",
    cw4GrpApTeam: "terra1jngs5xj00e9fq0hfmpr2pqyq96x3aj8la8kr3p",
    cw3ApTeam: "terra1yp5we2meetcfxql522q9ve3dsl29epye86528j",
    cw4GrpOwners: "terra1ldrkpnysrasq4sg4zu9mgh74wt9nxvk9qgvxtd",
    cw3GuardianAngels: "terra1ydp9qd9xgdq63ua6axfvauye3l7a3476lm6l28",
  },
  // TerraSwap balances account
  token_code_id: 23029,
  pair_code_id: 23028,
  factory_code_id: 23027,
  // LBP contracts
  factory_contract: "terra1wtj3tdf6jzdnkx52n805j3yz00p63l5rdcqnjv",
  token_contract: "terra149k0lmhudqzxx90htx7g5ka3wf5tt2peau3mpv",
  pair_contract: "terra1mvntqf2x3nzquws0j5n72jdqmyzg3eag6qlm7g",
  router_contract: "terra16csqc583xhxgz7pjuk7jvnr24pas0urg49tee6",

  // HALO contracts
  halo: {
    airdrop_contract: "terra1mndcdkklysq22h6rz4vdd3fvyx7tffc2swg0kc",
    collector_contract: "terra14d0uufjsmaktaanxtnlm38cullcq449gu35x0q",
    community_contract: "terra1peq4zcxzcpdv89vmfx4rvv4207esv6zdhqtfjd",
    distributor_contract: "terra13gjk2n3rhelgsnjhhe5jer9auvuqmcedzzclx9",
    gov_contract: "terra1l8aj2jvjsz3whqyvt0hz5fvf3vhd8gcrxf6r56",
    staking_contract: "terra1ve9jpy2pumfg33ky8gl7l6l3e7h8ppuwajkx0k",
    vesting_contract: "terra15scsvyuqkp0z2s8eet9p3jskduv8ypmc7ffwpn",
  }
} as const;

// ---------------------------------------------------------------------------------------------------
// MainNet information
// ---------------------------------------------------------------------------------------------------
export const mainnet = {
  // MainNet columbus-5
    networkInfo: {
    url: "https://apis.ankr.com/c29102fa57024dc5a5096bb73e7e0919/aae7334102f8f52264b50ad44bf343d3/terra/full/columbus",
    chainId: "columbus-5",
  },
  // MainNet MoneyMarket Contract
  anchorMoneyMarket: "terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s",
  // MainNet AP / DANO Treasury wallet (HALO collector contract)
  apTreasury: "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly",
  mnemonicKeys: {
    apTeam: "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
  },
  // Should be updated contract addresses after deploying wasms in the mainnet
  contracts: {
    registrar: "terra1nwk2y5nfa5sxx6gtxr84lre3zpnn7cad2f266h",
    indexFund: "terra19cevhng6nunl7gmc90sph0syuqyvtqn7mlhwz0",
    anchorVault: "terra172ue5d0zm7jlsj2d9af4vdff6wua7mnv6dq5vp",
    cw4GrpApTeam: "terra1qzdgs73h3rnh9j7f4t6tyuw9lmrp5esn6yunyl",
    cw3ApTeam: "terra1m6rqwmxlpztjf3zfhza906d44c9rpf2t6vn37n",
    cw4GrpOwners: "terra1lycc2zyhd676294c604euh8hxw7h6jrjd68x83",
    cw3GuardianAngels: "terra1jd2n0ze7er80x9h8k3x006aypaxs7mvrggdmn9",
    endowmentContracts: [
      "terra12crxq8nxml96e9h38fe67c4p76pc24l54zjzzh",
      "terra1uwtk2hs65332emnjn8n9s8d3l692pgyqnew4dq",
      "terra1qagm9wdnp6f76xy52llcjxmr4z8j4nhd9ethw8",
      "terra13nm3vyj6zfg2tdzsgq97ky6d6gtuty9mu025z3",
      "terra1d5phnyr7e7l44yaathtwrh4f4mv5agajcy508f",
      "terra1tkadaa8phaqnne20rzluhv8p57h23ku4n337ye",
      "terra18y4lflmg0wnlkw4hvamp4l2hjv2cafy7dtcyn6",
      "terra1c5luclcnzwhlf59c5w63yn034z6k9jrefx0swx",
      "terra1vqe93uv8lylkw4fc8m0xr89fv5xean29ftr0q2",
      "terra1k6v33x6sc9chztxgyh859npz740gmn9d4rnfkz",
      "terra1xmkprc4p2wxjh9eh58rjf3ndllepnl7xezmuk4",
      "terra1xmeept4tj37qqsajws8r6tl7f5hskvvfg2fmd5",
      "terra1zn8aqw3ypzvs8pzuadpqw5jw5rptxp4y08z7sr",
      "terra1cmp87658s0c475dkyee2p8r9zsdjd628py4zav",
      "terra1kdd6f099dv4kr5xqp7sxcc7epledxmvyq8xnu3",
      "terra16qh68y6gydhz73ndxgkzwmfcfam6jt45g8jhml",
      "terra1gm0x3m87e7wqzkac5eeacxkesy470yavpwqgdm",
      "terra1lalzy8rvkg3j2qm4a2x74lm6lxfa3llz7kdkdp",
      "terra1ngnqymglanujrhs09qakyz84k4v6cw9yyjwp0t",
      "terra1u0dls462h33j3fgg4j98wpa5qculnq2u749qc5",
      "terra17we0qdmf7gdnqrwtur52kn6898sf7tkxd8plq0",
      "terra1ex254ats3kd3t6tm7dj72e90vstvz59su2mvnq",
      "terra1fwkfncjwqnpw8snvtr20vjg3csqzeduznae628",
      "terra1cndqxysafnuvd2m7kd60vfh65qa4jdnx4l9p2f",
      "terra1c5kpr9pxnpfmznzhhz7cg7j5s0algnc8tk5kj6",
      "terra1d63nva4f7fdzlq5pnvs2wy3wkuh2qlaj77xpzr", // lay
      "terra19yy0g7jawqcfsh78e0zvywyagzezrjdhtuf94s", // foodbank
      "terra1aj240zlu6pg4yj6t2zqa6zu9dz5n5ez829cz22", // impact
      "terra1f5d62v4965gk3jvtffle6g722khgmt669dwmst", // childrenreach
      "terra1ah48pqeyrue2rrmfum094vqc3v7w7dhgy3zama", // riz
      "terra1c05fe2rpe0fs80m3nhuj952mt48amag75cyyu2", // Innocence Project of Texas
      "terra14gspqs563mv6q7v46s60exuc90w6u9k62wjq4s", // AfgFree
      "terra15jf75r43qrgr8xgppmzas7aelpsumpzvq6d8vl", // Marine Megafauna Foundation
      "terra10s24t8h939dtpqgvq77f23peam84ugv5n7y82r", // Isles
      "terra1yeqqexpkl230ca4kz4w88ne7vqzur4cncp3p5j", // 5Gyres
      "terra1d907r2n4j6k5xw7gw4qfphtlhvf0p36u2d8ydw", // CASD-SL
      "terra186zn38vk3qd405qgstr357cqu3p9axa2dr3fcf", // Civics Unplugged
      "terra1le244gz3ah64m25lxgemwwgnqlm3px8skwxx5r", // Ensemble Mik Nawooj
      "terra1wqx4frruqnn8nkeca7w35q34nd8g7p67tyk9n9", // The Institute for Citizens and Scholars
      "terra1s6n0aq4uqla263yjjn4ze8j544ephtqkrsach0", // For A Day Foundation
      "terra1xfecyvhwwet57fthejkle6a2suvehwwpeqzsj8", // Butte Country Local Food Network
      "terra1hs0aqqfq0qm2jjrdhxsuerk7hcs9n2yuuft072", // Power of Nutrition
      "terra1df9ek2ux6xka968jd5ze5ry5gj4zdwqy0z0jdu", // Ocean Web Alliance
    ],
  },
  members: [
    {addr: "terra1wvsugzhszkstexl0v6fv86c9ryjy8xm6u9t2fk", weight: 1},
    {addr: "terra103rakc90xgcuxaee6alqhkmnp7qh92hwt0hxur", weight: 1},
    {addr: "terra1numzqm5mgr56ftd4y8mfen7705nfs4vpz5jf0s", weight: 1},
    {addr: "terra1p3kcfzflagjl7lxfexwyaz43e4mprhyml0sqju", weight: 1},
  ],
  // TerraSwap balances account
  token_code_id: 3,
  pair_code_id: 4,
  factory_code_id: 5,

  // LBP contracts
  factory_contract: "terra1ulgw0td86nvs4wtpsc80thv6xelk76ut7a7apj",
  token_contract: "terra1w8kvd6cqpsthupsk4l0clwnmek4l3zr7c84kwq",
  pair_contract: "terra1s7kc5ua5936rfsqhd2rl0ngaphmegwcsze4txh",
  router_contract: "",

  // HALO contracts
  halo: {
    airdrop_contract: "",
    collector_contract: "",
    community_contract: "",
    distributor_contract: "",
    gov_contract: "",
    staking_contract: "",
    vesting_contract: "",
  }
} as const;
