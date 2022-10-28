import { IbcClient, Logger } from "@confio/relayer";
import { ChainDefinition, CosmWasmSigner, SigningOpts } from "@confio/relayer/build/lib/helpers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { stringToPath } from "@cosmjs/crypto";
import { Coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { localibc } from "../config/localIbcConstants";

type FundingOpts = SigningOpts & {
    readonly faucet: {
        readonly mnemonic: string;
    };
};

export interface AccountInfo {
    channel_id: string;
    last_update_time: string; // nanoseconds as string
    remote_addr?: string;
    remote_balance: Coin[];
}

/**
 * Return the `IbcClient` used for `IBC` connection setup.
 * @param opts SigingOpts
 * @param mnemonic 12 or 24 word mnemonic(string)
 * @param logger Logger
 * @returns `IbcClient`
 */
export async function customSigningClient(opts: SigningOpts, mnemonic: string, logger?: Logger): Promise<IbcClient> {
    let signer: DirectSecp256k1HdWallet;
    if (opts.prefix == "terra") {
        signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
            hdPaths: [stringToPath("m/44'/330'/0'/0/0")]
        })
    } else {
        signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
        });
    }
    const { address } = (await signer.getAccounts())[0];
    const options = {
        prefix: opts.prefix,
        gasPrice: GasPrice.fromString(opts.minFee),
        logger,
        estimatedBlockTime: opts.estimatedBlockTime,
        estimatedIndexerTime: opts.estimatedIndexerTime,
        ...extras(),
    };
    const client = await IbcClient.connectWithSigner(opts.tendermintUrlHttp, signer, address, options);
    return client;
}

/**
 * Return the `CosmwasmSigner` from `mnemonic`
 * @param opts SigingOpts
 * @param mnemonic String
 * @returns CosmwasmSigner
 */
export async function customSigningCosmWasmClient(opts: SigningOpts, mnemonic: string): Promise<CosmWasmSigner> {
    let wallet: DirectSecp256k1HdWallet;
    if (opts.prefix == "terra") {
        wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
            hdPaths: [stringToPath("m/44'/330'/0'/0/0")]
        })
    } else {
        wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: opts.prefix,
        });
    }
    const { address: senderAddress } = (await wallet.getAccounts())[0];
    const options = {
        prefix: opts.prefix,
        gasPrice: GasPrice.fromString(opts.minFee),
        ...extras(),
    };
    const sign = await SigningCosmWasmClient.connectWithSigner(opts.tendermintUrlHttp, wallet, options);
    return { sign, senderAddress };
}

/**
 * Fund the `rcpt` account from `faucet` account of chain
 * @param opts FundingOpts
 * @param rcpt Recipient address(string)
 * @param amount Amount of (fee) tokens(string)
 */
export async function customFundAccount(opts: FundingOpts, rcpt: string, amount: string) {
    const client = await customSigningClient(opts, opts.faucet.mnemonic);
    const feeTokens = {
        amount,
        denom: GasPrice.fromString(opts.minFee).denom,
    };
    await client.sendTokens(rcpt, [feeTokens]);
}

/**
 * Query the `ica_controller` contract for `list_accounts`.
 * @param cosmwasm CosmwasmSigner
 * @param controllerAddr `ica_controller` contract address
 * @returns Promise<AccountInfo[]>
 */
export async function listAccounts(cosmwasm: CosmWasmSigner, controllerAddr: string): Promise<AccountInfo[]> {
    const query = { list_accounts: {} };
    const res = await cosmwasm.sign.queryContractSmart(controllerAddr, query);
    return res.accounts;
}


/**
 * Exta fields used for `chain` configuration
 * @returns 2 fields `broadcastPollIntervalMs` & `broadcastTimeoutMs`
 */
function extras() {
    const extras = {
        // This is just for tests - don't add this in production code
        broadcastPollIntervalMs: 300,
        broadcastTimeoutMs: 5000,
    };
    return extras;
}

/**
 * Returns the pair of `IbcClient`s for setting up the IBC `Link`
 * @param srcConfig ChainDefinition
 * @param destConfig ChainDefinition
 * @returns [IbcClient, IbcClient]
 */
export async function setup(srcConfig: ChainDefinition, destConfig: ChainDefinition): Promise<[IbcClient, IbcClient]> {
    // create apps and fund an account
    const mnemonic = localibc.mnemonicKeys.signingClient;

    const src = await customSigningClient(srcConfig, mnemonic);
    const dest = await customSigningClient(destConfig, mnemonic);

    await customFundAccount(destConfig, dest.senderAddress, '4000000');
    await customFundAccount(srcConfig, src.senderAddress, '4000000');

    return [src, dest];
}
