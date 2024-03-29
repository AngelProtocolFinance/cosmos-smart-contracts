import { localjuno } from "./localjunoConstants";
import { localterra } from "./localterraConstants";

export const localibc = {
  mnemonicKeys: {
    signingClient:
      "charge strong album advance remind brain pool panic squeeze crystal pretty term remember power decorate lend pen ritual trick anxiety hat domain puzzle borrow",
  },

  config: {
    junoIcaController:
      "juno13nddjd7w2e0laxwsy3fjjhwyl5hczh0uv05v9jsv3y0um96ngfuqke0zzl",
    junoIcaHost:
      "juno16uumqeh9gvx9mhu2sfhqwrhsn8vvq95hxsu4aefdmm94p69s70aqflaskx",

    terraIcaController1:
      "terra1nwu2j6lfdff436xvuh4fggg687w0054a6l7uv2jmmwdwdtp5ht0sncrh8p",
    terraIcaController2:
      "terra1rh2yedhqylsqqc5wdrcxx7t5dcpaxmpyf4ugug4ep94k9lrw4gfqdnay4r",
    terraIcaHost:
      "terra1guc7sghs5gnsc44lrd0r9q0ey7akr94lz7ppyg4f6tp92vxmyulqsdwp25",
  },

  conns: {
    juno: "connection-6",
    terra: "connection-6",
  },

  channels: {
    junoAccount: "channel-17",
    terraVaultLocked: "channel-18",
    terraVaultLiquid: "channel-19",
  },

  contracts: {
    ibcVaultLocked1: "juno1...",
    ibcVaultLiquid1: "juno1...",
  },
};

export const junod = {
  tendermintUrlWs: "ws://localhost:26657",
  tendermintUrlHttp: "http://localhost:26657",
  chainId: "localjuno",
  prefix: "juno",
  denomStaking: "ujunox",
  denomFee: "ujuno",
  minFee: "0.025ujuno",
  blockTime: 250,
  faucet: {
    mnemonic: localjuno.mnemonicKeys.ast1,
    pubkey0: {
      type: "tendermint/PubKeySecp256k1",
      value: "A9cXhWb8ZpqCzkA8dQCPV29KdeRLV3rUYxrkHudLbQtS",
    },
    address0: "juno1qtajej3gturf3pmzgmrp92vuewltxug0284r8h",
  },
  ics20Port: "transfer",
  estimatedBlockTime: 400,
  estimatedIndexerTime: 80,
};

export const terrad = {
  tendermintUrlWs: "ws://localhost:26557",
  tendermintUrlHttp: "http://localhost:26557",
  chainId: "localterra",
  prefix: "terra",
  denomStaking: "uluna",
  denomFee: "uluna",
  minFee: "0.25uluna",
  blockTime: 250,
  faucet: {
    mnemonic: localterra.mnemonicKeys.test10,
    pubkey0: {
      type: "tendermint/PubKeySecp256k1",
      value: "A0d/GxY+UALE+miWJP0qyq4/EayG1G6tsg24v+cbD6By",
    },
    address0: "terra1fmcjjt6yc9wqup2r06urnrd928jhrde6gcld6n",
  },
  ics20Port: "transfer",
  estimatedBlockTime: 400,
  estimatedIndexerTime: 80,
};
