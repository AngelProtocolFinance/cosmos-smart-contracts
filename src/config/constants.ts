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
  // TestNet AP / DANO Treasury wallet
  apTreasury: "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly",
  mnemonicKeys: {
    apTeam:
      "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
    apTeam2:
      "custom review state crisp modify sell trick replace bone wolf ridge paper later collect topple income owner head turkey estate canyon tone copy inhale",
    apTeam3:
      "law cause body surround problem join swift shy lumber start immense spray mandate organ pledge butter modify fossil pluck demise link bus rebel misery",
    charity1:
      "source multiply curtain modify nurse party valid awesome road local focus retreat route agree spot rule false cloud dwarf six relief clay unhappy thank",
    charity2:
      "stick dumb cabin wish great impact fork save trade crime today seed tortoise base enter topic physical glue maple cliff over myth marble loyal",
    charity3:
      "write obscure shop lunar fruit attend media abuse spirit lens illegal pluck rally cave stamp gadget burger rigid minute index paper voice eight again",
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
    cw4GrpApTeam: "terra1wpnzy6w9gd3tt9wkvnqkcmzkyc8v0tgz75nuue",
    cw3ApTeam: "terra1qspgamxqn9slwe7ecca4n2fs2xsl5hxvkc9lzs",
    cw4GrpOwners: "terra1ldrkpnysrasq4sg4zu9mgh74wt9nxvk9qgvxtd",
    cw3GuardianAngels: "",
  },

  // LBP contracts
  lbp: {
    factory_contract: "terra167m64seqj7cucxm5wep3hyu4suqw4sl5s8uzjz",
    router_contract: "terra19dpanzuhtmdsw8ds5zschrh4mnxcejc0ut6dnk",
    pair_contract: "terra17al3hudq2vcxtyvw9008edjhyqaw74mayva2d8",
    lp_token_contract: "terra19zgdunfrx79nqvznqmx4satj5kxndvmrsx502m",
    // HALO/UST Pair token supply amount
    halo_token_amount: "80000000000",
    native_token_amount: "1300000000", // adjusted down from localterra/mainnet values due to faucet limitations
    // HALO/UST Pair start/end times
    lbp_start_time: "2021-12-18T02:04:00.000Z",
    lbp_end_time: "2021-12-19T02:04:00.000Z",
    // HALO/UST Pair start/end weights
    token_start_weight: "96",
    token_end_weight: "50",
    native_start_weight: "4",
    native_end_weight: "50",
    slippage_tolerance: "0.01",
  },

  // TerraSwap contracts
  terraswap: {
    terraswap_token_code: 148,
    terraswap_factory: "terra18qpjm4zkvqnpjpw0zn0tdr8gdzvt8au35v45xf",
    halo_token_contract: "terra1ah3gd4uhxtrpc3qeqn84l6v3wcvfkh3vw25fyl",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "20000000000",
    native_liquidity: "1000000000", // reduced to 1000 UST due to faucet limitations
  },

  // HALO contracts
  halo: {
    airdrop_contract: "terra15n2j80ufyrup8ply5nhjwwerfjpz7cx3m2hcqq",
    collector_contract: "terra12h9ssarf78t3sw3zu5xyr4v3cxjkmqmnxujlzl",
    community_contract: "",
    distributor_contract: "terra1vuktqwu0n5df0sfswzwwh4wpgznu4d2urflvlk",
    gov_contract: "terra16tw444h6qtzxr4kf2p276qt0u6w3ggtc20xgly",
    gov_hodler_contract: "terra1mcjrurlzmne3hlqvjypyacz9l8xpf4r6zq9sa6",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;

// ---------------------------------------------------------------------------------------------------
// MainNet information
// ---------------------------------------------------------------------------------------------------
export const mainnet = {
  // MainNet columbus-5
  networkInfo: {
    url: "https://lcd.terra.dev",
    chainId: "columbus-5",
  },
  // MainNet MoneyMarket Contract
  anchorMoneyMarket: "terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s",
  // MainNet AP / DANO Treasury wallet
  apTreasury: "terra1rcznds2le2eflj3y4e8ep3e4upvq04sc65wdly",
  mnemonicKeys: {
    apTeam:
      "forward stone width wrist outer elder supply summer extra erosion spring unlock rhythm sail goose once city ivory eight diesel upper measure betray purchase",
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
      "terra1rttuqe9dsf5hksep8syv8nelrmdlyv2pnrx2w8", // Project Hawaii
      "terra1p2dq0ct6xlt05y2zxhjfqt0nl3awp57yxdk9mq", // Cup of Hope Las Vegas
      "terra16h3qzecumpa5lxf6ekt2869mpycms5rac0lwp8", // Sumarth
      "terra1yfemvj4epgx74j8jm0gfl3n2qen2w9q6eyhan8", // Imagine Worldwide
      "terra1y7rk4v0y58lr2w0sx5jy2n3yl046k9yfllk2p6", // Wayuu Taya
      "terra1k67vqfnycwzgtucz27y4z3wxqr5shdl7rx3a0w", // Eagle's Nest
      "terra160qenex0dhdrms3lzkjv0jj26v8vkekptl5egf", // Outshine LGBTQ Film Festival
      "terra1vm7g8ah6v95xs4d8q774fhavfrc5f2lzf9fygs", // Alex's Lemonade Stand Foundation
      "terra1m6w3mk063vglg6eyazf8llm40wcsjsdg29sm7w", // The Biodiversity Group
      "terra12854vdzzr909ss24lqc8nsqnvg378l23p9vukr", // SOWE
      "terra1sxpz8mm4kcsz8rg60436d3z2td6v76qxnfj056", // Papyrus
      "terra1hs65s48t3f7rj8z4cldst97wnsd8cchjzt4tc2", // BLOC
      "terra1s5as2etdefsgvz6aju5wctutaqyurgc034d2ss", // MADEMAN Foundation
      "terra1pvjzgywze624p9u373c6jfvgda5plnrgxy7tw8", // Water for Africa
      "terra1pgzqqpqy25vqq5q9wlgzujz6jhx87megmcqlgz", // Ogallala Life
      "terra14jx04p7wzjjnrjwaslgctwkqn5dh56upexvy5l", // Music 4 Peace Foundation
      "terra1r3fga9xmf7wd6y38vzljqqa40juaeuzjrt05fx", // Black Women Network
      "terra1mcf9lhce23znkpmvg6c5pxx0a36s03yamsklad", // Walk Church
      "terra18dpzlvl78a48fjrfup6y9e3m0wjnylsa6pnaey", // 9 Lives Project
      "terra1w4q8j6hgun0zz6smqnt5n7zv23e8wsedd3zwjk", // Silvermine Arts Center
      "terra1m90s9m4xn5rv45t3d7d4k4xzcckgeheg37gq33", // Circle of Care
      "terra12ws5eecm3em6f9n6gj354lf8nc7jf40mq0aepc", // Waveny Lifecare Network
      "terra1ju737ylc3w9ltk5p643ts8k04mc5ncx4a7zxju", // Start Rescue
      "terra1eezt4at6jmg762faf6sk55tktqgl6ml4zqywv0", // Yamba Hearts
      "terra1m7sf34mp05hea0t7x2huye36u3yq7czn6g87el", // Dogs Cleaveland
      "terra1ljhyxwjqngxjc6x85e09l658xrduqyurfcfr0n", // Wikitongues
      "terra15kjfh3rz9yn9hwzcp9y4nke5k2q3x74tactu0g", // Hip Hop for Change
      "terra1l8lu43gmlnv6plcp0nxhuk9jkw6gy9nr8kng7n", // Empower through Health
      "terra1dp3zw599k3h3n7qsqzell3fnanmw65n2s3ddwt", // Associazione Teia
      "terra1vmmkdcw49gt8guv7su4fkksyv6dnthafeap4fw", // Rose Knight Foundation
      "terra15w8pfr48vxzparmf8qvwsxtlkzkf0uvawzxaeu", // CORE (Community Organized Relief Effort)
      "terra1ysuujvuhefphh6ktuawvlgpzewye7stczxzaht", // Taking Pictures, Changing Lives Foundation
      "terra1zv52rlllk2vemgm5jk77agpukk5zqk5fmgkf2t", // REEFolution
      "terra1uzrw4d95dyh5d4mygnzmgfyrushdygdecepcmf", // Proyecto Guajira
      "terra1zjvygqzen4ctsjvre507vggexgtrlsal5q5nen", // Pangea Educational Development
      "terra14ykrrzsvnjnf7664asf988p7snn33kxc5hsyju", // Direct Hand Foundation
      "terra1vuqcqjs684sdg8gw44zr8h57tethxsz677kl79", // Chosen Generations Community Center
      "terra15eyed7vc0apncafs7axdngn3e9tlx2zg06jyax", // The Society Library
      "terra1hpdlddtyelyggzw2kteqwu8kclma0was7paedz", // Dream Machine Foundation
      "terra1lnnu4rxzy6grumrrzu4jnq2vcsxnwzxqkrrlqe", // Upstream
      "terra1xwepmhuqe2w7huh4nfq2ljc62ugchqe9z2q6hy", // Tibet Relief Fund
      "terra1ku9qpt8ym2tyqutmqvfqh83m7mqu9f8sm26zc3", // America Developing Smiles
      "terra1gvr02p5k77zfztzd6zavp34n8tyljlqquls7z8", // Karma Delivers
      "terra1cle32fewy2xdh9z7auzj54zwjyq29tdyr20w3c", // 7000 languages
      "terra15xturl8f6jf5p3m7u4ln32yxz76f9d4l9fnjmt", // Homen Novo
      "terra19ncyspupfedqkfhckpmegtzjfpk7yae76smgtc", // Human Needs Project
      "terra13myz0spjjzz3vkep5ac4mtzcffxc0rth4ea2lm", // Cat Rescue
      "terra1q4sjzkztrpfujqu5vzquhvhvqy872d0drcfuq4", // Legaler Aid
      "terra15a024cu3pmzlcu3de4jqcrws0qn3ta6l6q2h4c", // Fundación CONIN
      "terra10t7tqx5gfc5msjwvvkvga7v6yqdslj67mgh8g0", // Notes For Notes
      "terra1c3jyn8pw2vuh9qsync50kgj2pu2aea5page3r3", // mothers2mothers
      "terra1jcnz99mhmg3zdgxcjs2j8h66emqrc7l3emrq90", // Earth Codes Observatory
      "terra1ypzgt3zjl6uzadvtnza25vr687ecry09spfqc9", // Team Up Canada
      "terra10lttp7re6nscetk34t547lrnudeeull3xha0k6", // PMDP Hawaii
      "terra18lstx69vt9799h6uywltze30487rp9lnhn0vve", // Blue Frontier
      "terra19lldf03f8fyr4lcut6rn5c8xd3zr8kgpggxcn4", // Congo Biotropical Institute
      "terra1y27umc27wg9vtx5jadj5q0qu7yw7klys74yy08", // ISERA
      "terra1vkrjvqxw33zdf5m67x8c2elm44xwcw0jk6fmj7", // Marrvelous Pet Rescues
      "terra1yt9wghlqem8gql664mjfqwwe636g3t0xypf9lk", // Seminole Indian Scouts Cemetary Association
      "terra10qeaeukmyy2kqw5f2fvut5pmm9gaqvnrqtsk22", // Third Wave Volunteers
      "terra1c95a3sjpt3jat2azphjt2sk8xqvcwj0uexjp8d", // Nova Ukraine
      "terra14s30dkddgzu86s5yudahdm0xa9ks3u8xuht7rc", // GSG
      "terra10gw4kgx8myfsmyr5mz472uvvseedgx7k9z5pv5", // Simbi Community Development
      "terra1f5rtva4yjtqcr5t9dlpatgqnk0e08s5s5a3sal", // Global Synergy for Leadership
      "terra1x9cshctwn06v7gzj6sgkhpmf4tsz5d83rrak5a", // Turtle Foundation
      "terra1xt696t8ldx94cdrwmtznn7g2uzwfeldwclyq03", // Bumi Sehat Foundation International
      "terra1k5auhclwzt0yy6gq5e4h9kzk8zxqgaetu9cvna", // Microscopio
      "terra1hccjcxm0vdz8d2n9y8lnrpx4ka4elt4gwfm522", // Threshold
    ],
  },
  members: [
    { addr: "terra1wvsugzhszkstexl0v6fv86c9ryjy8xm6u9t2fk", weight: 1 },
    { addr: "terra103rakc90xgcuxaee6alqhkmnp7qh92hwt0hxur", weight: 1 },
    { addr: "terra1numzqm5mgr56ftd4y8mfen7705nfs4vpz5jf0s", weight: 1 },
    { addr: "terra1p3kcfzflagjl7lxfexwyaz43e4mprhyml0sqju", weight: 1 },
  ],

  // LBP contracts
  lbp: {
    factory_contract: "terra10dl5u40lj50scntv4qmwykfw2zulf77zyv34u0",
    router_contract: "terra1l32eafhapmn9c8m7epyraxa2yty4xngamvewfs",
    pair_contract: "terra1hhpgcp2stvzx952zfxtxg4dhgf60yfzchesj3e",
    lp_token_contract: "terra1kt26adtzwu4yefw37snr73n393vsu8w0hmazxc",
    // HALO/UST Pair token supply amount
    halo_token_amount: "80000000000000",
    native_token_amount: "1300000000000",
    // HALO/UST Pair start/end times
    lbp_start_time: "2021-12-19T15:00:00.000Z",
    lbp_end_time: "2021-12-22T15:00:00.000Z",
    // HALO/UST Pair start/end weights
    token_start_weight: "96",
    token_end_weight: "50",
    native_start_weight: "4",
    native_end_weight: "50",
    slippage_tolerance: "0.01",
  },

  // TerraSwap contracts
  terraswap: {
    terraswap_token_code: 3,
    terraswap_factory: "terra1ulgw0td86nvs4wtpsc80thv6xelk76ut7a7apj",
    halo_token_contract: "terra1w8kvd6cqpsthupsk4l0clwnmek4l3zr7c84kwq",
    halo_ust_pair_contract: "",
    halo_ust_pair_lp_token: "",
    initial_halo_supply: "1000000000000000",
    halo_liquidity: "",
    native_liquidity: "",
  },

  // LOOP LP contracts
  loop: {
    terraswap_factory: "terra16hdjuvghcumu6prg22cdjl96ptuay6r0hc6yns",
    pair_contract: "terra1yjg0tuhc6kzwz9jl8yqgxnf2ctwlfumnvscupp",
    lp_token_contract: "terra17pzt8t2hmx6587zn6yh5ensylm3s9mm4m72v2n",
    // terra12aazc56hv7aj2fcvmhuxve0l4pmayhpn794m0p /// HALO-LOOP PAIR
  },

  // HALO contracts
  halo: {
    airdrop_contract: "terra1pe6mnf0ursz0h80h2hwk690hvrph8vgt9pnw0w",
    collector_contract: "terra1uxqjsgnq30lg5lhlhwd2gmct844vwqcdlv93x5",
    community_contract: "",
    distributor_contract: "terra1ya34r8qj0fttkrxx435zexshyqe5fe3vlmhnd6",
    gov_contract: "terra1zcmp45vemypvd3j6ek2j2gz4mevjzyv3jc4ree",
    gov_hodler_contract: "terra1vn8ycrkmm8llqcu82qe3sg5ktn6hajs6tkpnx0",
    staking_contract: "",
    vesting_contract: "",
  },
} as const;
