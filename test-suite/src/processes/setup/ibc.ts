// LocalJuno-related imports
import { Link, Logger } from "@confio/relayer";
import {
  ChainDefinition,
  CosmWasmSigner,
} from "@confio/relayer/build/lib/helpers";
import {
  LCDClient,
  LocalTerra,
  MnemonicKey,
  MsgExecuteContract,
  Wallet,
} from "@terra-money/terra.js";
import chalk from "chalk";
import { junod, terrad } from "../../config/localIbcConstants";
import { localjuno } from "../../config/localjunoConstants";
import { localterra } from "../../config/localterraConstants";

import { wasm_path } from "../../config/wasmPaths";
import { customSigningCosmWasmClient, setup } from "../../utils/ibc";
import {
  instantiateContract,
  sendMessageViaCw3Proposal,
  storeCode,
} from "../../utils/juno/helpers";

import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";

// LocalTerra-related imports
import {
  instantiateContract as tInstantiateContract,
  storeCode as tStoreCode,
  sendTransaction as tSendTransaction,
} from "../../utils/terra/helpers";

// -------------------------------------------------------------------------------------
// Variables
// -------------------------------------------------------------------------------------
let junoIbcSetupper: CosmWasmSigner;
let junoIcaController: string;
let junoIcaControllerPort: string;
let junoIcaHost: string;
let junoIcaHostPort: string;

let terra: LocalTerra | LCDClient;
let terraIbcSetupper: Wallet;

let terraIcaController1: string;
let terraIcaController1Port: string;
let terraIcaController2: string;
let terraIcaController2Port: string;
let terraIcaHost: string;
let terraIcaHostPort: string;

let channelId0: string;
let channelId1: string;
let channelId2: string;

let junoTransferChannel: string;
let terraTransferChannel: string;

const IbcVersion = "ica-vaults-v1";
// const Ics20Version = "ics20-1";

// -------------------------------------------------------------------------------------
// setup all contracts for LocalJuno and TestNet
// -------------------------------------------------------------------------------------
export async function setupIBC(): Promise<void> {
  junoIbcSetupper = await customSigningCosmWasmClient(
    junod,
    localjuno.mnemonicKeys.ast1
  );
  await deployJunoIcaContracts();

  terra = new LCDClient({
    URL: localterra.networkInfo.url,
    chainID: localterra.networkInfo.chainId,
  });
  terraIbcSetupper = new Wallet(
    terra,
    new MnemonicKey({ mnemonic: localterra.mnemonicKeys.test10 })
  );
  await deployTerraIcaContracts();

  await customConnSetup(junod, terrad);

  await postProcess();

  console.log(chalk.green(" Done!"));
}

async function deployJunoIcaContracts(): Promise<void> {
  // Step 1: Upload the wasms
  process.stdout.write("Uploading ica_controller wasm on JUNO");
  const icaControllerCodeId = await storeCode(
    junoIbcSetupper.sign,
    junoIbcSetupper.senderAddress,
    `${wasm_path.core}/ica_controller.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${icaControllerCodeId}`
  );

  process.stdout.write("Uploading ica_host wasm on JUNO");
  const icaHostCodeId = await storeCode(
    junoIbcSetupper.sign,
    junoIbcSetupper.senderAddress,
    `${wasm_path.core}/ica_host.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${icaHostCodeId}`
  );

  process.stdout.write("Uploading cw1_whitelist wasm on JUNO");
  const cw1WhitelistCodeId = await storeCode(
    junoIbcSetupper.sign,
    junoIbcSetupper.senderAddress,
    `${wasm_path.cwPlus}/cw1_whitelist.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`
  );

  // Step 2: JunoIbcClient set up the "ica"-related contracts
  process.stdout.write("Instantiating (juno)ica_controller contract");
  const icaControllerResult = await instantiateContract(
    junoIbcSetupper.sign,
    junoIbcSetupper.senderAddress,
    junoIbcSetupper.senderAddress,
    icaControllerCodeId,
    {}
  );
  junoIcaController = icaControllerResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue(" contractAddress")}=${junoIcaController}`
  );

  let contractQuery = await junoIbcSetupper.sign.getContract(junoIcaController);
  junoIcaControllerPort = contractQuery.ibcPortId || "";

  process.stdout.write("Instantiating (juno)ica_host contract");
  const icaHostResult = await instantiateContract(
    junoIbcSetupper.sign,
    junoIbcSetupper.senderAddress,
    junoIbcSetupper.senderAddress,
    icaHostCodeId,
    {
      cw1_code_id: cw1WhitelistCodeId,
    }
  );
  junoIcaHost = icaHostResult.contractAddress as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue(" contractAddress")}=${junoIcaHost}`
  );

  contractQuery = await junoIbcSetupper.sign.getContract(junoIcaHost);
  junoIcaHostPort = contractQuery.ibcPortId || "";
}

async function deployTerraIcaContracts(): Promise<void> {
  // Step 1: Upload the wasms
  process.stdout.write("Uploading ica_controller wasm on TERRA");
  const icaControllerCodeId = await tStoreCode(
    terra,
    terraIbcSetupper,
    `${wasm_path.core}/ica_controller.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${icaControllerCodeId}`
  );

  process.stdout.write("Uploading ica_host wasm on TERRA");
  const icaHostCodeId = await tStoreCode(
    terra,
    terraIbcSetupper,
    `${wasm_path.core}/ica_host.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${icaHostCodeId}`
  );

  process.stdout.write("Uploading cw1_whitelist wasm on TERRA");
  const cw1WhitelistCodeId = await tStoreCode(
    terra,
    terraIbcSetupper,
    `${wasm_path.cwPlus}/cw1_whitelist.wasm`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("codeId")}=${cw1WhitelistCodeId}`
  );

  // Step 2: TerraIbcClient set up the "ica"-related contracts
  process.stdout.write(
    "Instantiating (terra)ica_controller1 contract(for locked vault)"
  );
  const res1 = await tInstantiateContract(
    terra,
    terraIbcSetupper,
    terraIbcSetupper,
    icaControllerCodeId,
    {}
  );
  terraIcaController1 = res1.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue(" contractAddress")}=${terraIcaController1}`
  );

  let contractQuery = await terra.wasm.contractInfo(terraIcaController1);
  terraIcaController1Port = contractQuery.ibc_port_id || "";

  process.stdout.write(
    "Instantiating (terra)ica_controller2 contract(for liquid vault)"
  );
  const res2 = await tInstantiateContract(
    terra,
    terraIbcSetupper,
    terraIbcSetupper,
    icaControllerCodeId,
    {}
  );
  terraIcaController2 = res2.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue(" contractAddress")}=${terraIcaController2}`
  );

  contractQuery = await terra.wasm.contractInfo(terraIcaController2);
  terraIcaController2Port = contractQuery.ibc_port_id || "";

  process.stdout.write("Instantiating (terra)ica_host contract");
  const icaHostResult = await tInstantiateContract(
    terra,
    terraIbcSetupper,
    terraIbcSetupper,
    icaHostCodeId,
    {
      cw1_code_id: cw1WhitelistCodeId,
    }
  );
  terraIcaHost = icaHostResult.logs[0].events
    .find((event) => {
      return event.type == "instantiate";
    })
    ?.attributes.find((attribute) => {
      return attribute.key == "_contract_address";
    })?.value as string;
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue(" contractAddress")}=${terraIcaHost}`
  );

  contractQuery = await terra.wasm.contractInfo(terraIcaHost);
  terraIcaHostPort = contractQuery.ibc_port_id || "";
}

async function postProcess() {
  process.stdout.write("Updating admins of controller & host contracts");
  await junoIbcSetupper.sign.execute(
    junoIbcSetupper.senderAddress,
    junoIcaController,
    {
      update_admin: {
        admin: localjuno.contracts.accounts,
      },
    },
    "auto"
  );

  await tSendTransaction(terra, terraIbcSetupper, [
    new MsgExecuteContract(
      terraIbcSetupper.key.accAddress,
      terraIcaController1,
      {
        update_admin: {
          admin: localterra.contracts.vaultLocked1,
        },
      }
    ),
  ]);

  await tSendTransaction(terra, terraIbcSetupper, [
    new MsgExecuteContract(
      terraIbcSetupper.key.accAddress,
      terraIcaController2,
      {
        update_admin: {
          admin: localterra.contracts.vaultLiquid1,
        },
      }
    ),
  ]);
  console.log(chalk.green(" Done!"));

  process.stdout.write(
    "Updating configs of `(juno) accounts` and `(terra) vault` contracts"
  );
  const junoAPTeamSigner = await customSigningCosmWasmClient(
    junod,
    localjuno.mnemonicKeys.apTeam
  );
  await sendMessageViaCw3Proposal(
    junoAPTeamSigner.sign,
    junoAPTeamSigner.senderAddress,
    localjuno.contracts.cw3ApTeam,
    localjuno.contracts.accounts,
    {
      update_config: {
        new_registrar: localjuno.contracts.registrar,
        max_general_category_id: 1,
        ibc_controller: junoIcaController,
      },
    }
  );

  const terraApTeamSigner = await customSigningCosmWasmClient(
    terrad,
    localterra.mnemonicKeys.test1
  ); // `test1` wallet is used as `APTeam` wallet.
  const accountInfo0 = await terraApTeamSigner.sign.queryContractSmart(
    terraIcaHost,
    {
      account: {
        channel_id: channelId0,
      },
    }
  );
  await terraApTeamSigner.sign.execute(
    terraApTeamSigner.senderAddress,
    localterra.contracts.vaultLocked1,
    {
      update_config: {
        ibc_host: accountInfo0.account,
        ibc_controller: terraIcaController1,
      },
    },
    "auto"
  );
  await terraApTeamSigner.sign.execute(
    terraApTeamSigner.senderAddress,
    localterra.contracts.vaultLiquid1,
    {
      update_config: {
        ibc_host: accountInfo0.account,
        ibc_controller: terraIcaController2,
      },
    },
    "auto"
  );

  process.stdout.write(
    "Register the ibc Terra link info(NetworkInfo) to the (juno)Registrar contract"
  );
  await sendMessageViaCw3Proposal(
    junoAPTeamSigner.sign,
    junoAPTeamSigner.senderAddress,
    localjuno.contracts.cw3ApTeam,
    localjuno.contracts.registrar,
    {
      update_network_connections: {
        action: "post",
        network_info: {
          name: "Terra",
          chain_id: localterra.networkInfo.chainId,
          ibc_channel: channelId0,
          transfer_channel: junoTransferChannel,
          ibc_host_contract: junoIcaHost,
          gas_limit: undefined,
        },
      },
    }
  );

  process.stdout.write(
    "Register the (terra)vaults to the (juno)registrar contract"
  );
  const accountInfo1 = await junoAPTeamSigner.sign.queryContractSmart(
    junoIcaHost,
    {
      account: {
        channel_id: channelId1,
      },
    }
  );
  await sendMessageViaCw3Proposal(
    junoAPTeamSigner.sign,
    junoAPTeamSigner.senderAddress,
    localjuno.contracts.cw3ApTeam,
    localjuno.contracts.registrar,
    {
      vault_add: {
        network: localterra.networkInfo.chainId,
        vault_addr: accountInfo1.account,
        input_denom: localterra.denoms.usdc,
        yield_token: localjuno.contracts.registrar, // Really needed?
        restricted_from: [],
        acct_type: `locked`,
        vault_type: { ibc: { ica: localterra.contracts.vaultLocked1 } },
      },
    }
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Terra Vault1(Locked) - contractAddress")}=${
      accountInfo1.account
    }`
  );

  const accountInfo2 = await junoAPTeamSigner.sign.queryContractSmart(
    junoIcaHost,
    {
      account: {
        channel_id: channelId2,
      },
    }
  );
  await sendMessageViaCw3Proposal(
    junoAPTeamSigner.sign,
    junoAPTeamSigner.senderAddress,
    localjuno.contracts.cw3ApTeam,
    localjuno.contracts.registrar,
    {
      vault_add: {
        network: localterra.networkInfo.chainId,
        vault_addr: accountInfo2.account,
        input_denom: localterra.denoms.usdc,
        yield_token: localjuno.contracts.registrar, // Really needed?
        restricted_from: [],
        acct_type: `liquid`,
        vault_type: { ibc: { ica: localterra.contracts.vaultLiquid1 } },
      },
    }
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("Terra Vault1(Liquid) - contractAddress")}=${
      accountInfo2.account
    }`
  );
}

/**
 * Clone of original "@confio/relayer/testutils/setup" util.
 * create a connection and channel for simple-ica
 * @param srcConfig Source chain definition
 * @param destConfig Destination chain definition
 * @param logger
 * @returns IBC link
 */
async function customConnSetup(
  srcConfig: ChainDefinition,
  destConfig: ChainDefinition,
  logger?: Logger
) {
  process.stdout.write("Setting up the IBC connection\n");

  // Setup the IbcClients
  const [src, dest] = await setup(srcConfig, destConfig);

  // Setup ibc link/connection
  const link = await Link.createWithNewConnections(
    src,
    dest,
    undefined,
    599,
    599
  ); /// "TrustPeriod"s should be < 10 mins(600s).
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("conns-juno(connA)")}=${link.endA.connectionID}`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("conns-terra(connB)")}=${link.endB.connectionID}`
  );

  // Create channels between controller & host contracts(ports)
  const channel0 = await link.createChannel(
    "A",
    junoIcaControllerPort,
    terraIcaHostPort,
    Order.ORDER_UNORDERED,
    IbcVersion
  );
  const channel1 = await link.createChannel(
    "B",
    terraIcaController1Port,
    junoIcaHostPort,
    Order.ORDER_UNORDERED,
    IbcVersion
  );
  const channel2 = await link.createChannel(
    "B",
    terraIcaController2Port,
    junoIcaHostPort,
    Order.ORDER_UNORDERED,
    IbcVersion
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("JUNO cont -> TERRA host channelId")}=${
      channel0.src.channelId
    }, ${channel0.dest.channelId}`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("TERRA cont1 -> JUNO host channelId")}=${
      channel1.src.channelId
    }, ${channel1.dest.channelId}`
  );
  console.log(
    chalk.green(" Done!"),
    `${chalk.blue("TERRA cont2 -> JUNO host channelId")}=${
      channel2.src.channelId
    }, ${channel2.dest.channelId}`
  );
  channelId0 = channel0.src.channelId;
  channelId1 = channel1.src.channelId;
  channelId2 = channel2.src.channelId;

  // also create a ics20 channel on this connection
  const ics20Info1 = await link.createChannel(
    "A",
    junod.ics20Port,
    terrad.ics20Port,
    Order.ORDER_UNORDERED,
    "ics20-1"
  );
  const ics20Info2 = await link.createChannel(
    "B",
    terrad.ics20Port,
    junod.ics20Port,
    Order.ORDER_UNORDERED,
    "ics20-1"
  );
  // const ics20 = {
  //     juno: ics20Info1.src.channelId,
  //     terra: ics20Info2.src.channelId,
  // };
  junoTransferChannel = ics20Info1.src.channelId as string;
  terraTransferChannel = ics20Info2.src.channelId as string;
  console.log(junoTransferChannel);
  console.log(terraTransferChannel);

  await link.relayAll();

  return { link };
}
