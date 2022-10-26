
export const localibc = {
    mnemonicKeys: {
        junoIbcClient: "soda tomato draft between amazing grab suit verb help pony elegant oxygen trial cactus coffee",
        terraIbcClient: "punch relax wire approve nice sting cargo explain inside acid people achieve later owner once organ mountain ripple ankle chapter holiday since aspect scrub",

        signingClient: "charge strong album advance remind brain pool panic squeeze crystal pretty term remember power decorate lend pen ritual trick anxiety hat domain puzzle borrow",
    },
}

export const junod = {
    tendermintUrlWs: 'ws://localhost:26657',
    tendermintUrlHttp: 'http://localhost:26657',
    chainId: 'localjuno',
    prefix: 'juno',
    denomStaking: 'ujunox',
    denomFee: 'ujuno',
    minFee: '0.025ujuno',
    blockTime: 250,
    faucet: {
        mnemonic: localibc.mnemonicKeys.junoIbcClient,
        pubkey0: {
            type: 'tendermint/PubKeySecp256k1',
            value: 'A9cXhWb8ZpqCzkA8dQCPV29KdeRLV3rUYxrkHudLbQtS',
        },
        address0: 'juno1n8y753tnrv75dlmlnyex4h9k84jrmejycc3rxy',
    },
    ics20Port: 'transfer',
    estimatedBlockTime: 400,
    estimatedIndexerTime: 80,
};

export const terrad = {
    tendermintUrlWs: 'ws://localhost:26557',
    tendermintUrlHttp: 'http://localhost:26557',
    chainId: 'localterra',
    prefix: 'terra',
    denomStaking: 'uluna',
    denomFee: 'uluna',
    minFee: '0.25uluna',
    blockTime: 250,
    faucet: {
        mnemonic: localibc.mnemonicKeys.terraIbcClient,
        pubkey0: {
            type: 'tendermint/PubKeySecp256k1',
            value: 'A0d/GxY+UALE+miWJP0qyq4/EayG1G6tsg24v+cbD6By',
        },
        address0: 'terra10ldxyk6vcupuxlugnec2ugyddy4558062cc0y9',
    },
    ics20Port: 'transfer',
    estimatedBlockTime: 400,
    estimatedIndexerTime: 80,
};