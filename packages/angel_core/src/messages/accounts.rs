use crate::structs::{
    AccountType, Beneficiary, Categories, DaoSetup, EndowmentFee, EndowmentType, RebalanceDetails,
    SettingsController, SplitDetails, StrategyComponent, SwapOperation,
};
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw4::Member;
use cw_asset::{Asset, AssetInfo, AssetUnchecked};
use cw_utils::{Expiration, Threshold};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub last_earnings_harvest: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    /// reinvest vault assets from Liquid to Locked
    ReinvestToLocked {
        id: u32,
        amount: Uint128,
        vault_addr: String,
    },
    // Pull funds from Endowment locked/liquid free balances (TOH) to a Beneficiary address
    Withdraw {
        id: u32,
        acct_type: AccountType,
        beneficiary: String,
        assets: Vec<AssetUnchecked>,
    },
    SwapToken {
        id: u32,
        acct_type: AccountType,
        amount: Uint128,
        operations: Vec<SwapOperation>,
    },
    // Router notifies the Accounts of final tokens from a Swap
    // Allows Accounts to credit the Endowment's involved Balance
    // with the amount returned to the main Accounts contract
    SwapReceipt {
        id: u32,
        acct_type: AccountType,
        final_asset: Asset,
    },
    // Tokens are sent back to an Account from an Asset Vault
    VaultReceipt {
        id: u32,
        acct_type: AccountType,
    },
    // Invest TOH funds into Vaults
    VaultsInvest {
        id: u32,
        acct_type: AccountType,
        vaults: Vec<(String, Asset)>,
    },
    // Redeem TOH funds from Vaults
    VaultsRedeem {
        id: u32,
        acct_type: AccountType,
        vaults: Vec<(String, Uint128)>,
    },
    // create a new endowment
    CreateEndowment(CreateEndowmentMsg),
    // Winding up / closing of an endowment. Returns all funds to a specified Beneficiary address if provided.
    // If not provided, looks up the Index Fund an Endowment is tied to to donates the funds to it.
    CloseEndowment {
        id: u32,
        beneficiary: Beneficiary,
    },
    DistributeToBeneficiary {
        id: u32,
    },
    // update owner addrInstantiateMsg
    UpdateOwner {
        new_owner: String,
    },
    // update config
    // Allows the SC owner (only!) to change ownership & upper limit of general categories ID allowed
    UpdateConfig(UpdateConfigMsg),

    // Update an Endowment owner, beneficiary, and other settings
    UpdateEndowmentSettings(UpdateEndowmentSettingsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
    // Replace an Account's Strategy with that given.
    UpdateStrategies {
        id: u32,
        acct_type: AccountType,
        strategies: Vec<Strategy>,
    },
    // Manage the allowances for the 3rd_party wallet to withdraw
    // the endowment TOH liquid balances without the proposal
    Allowance {
        endowment_id: u32,
        action: String,
        spender: String,
        asset: Asset,
        expires: Option<Expiration>,
    },
    // Withdraws the free TOH liquid balances of endowment by 3rd_party wallet
    SpendAllowance {
        endowment_id: u32,
        asset: Asset,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub new_registrar: String,
    pub max_general_category_id: u8,
    pub ibc_controller: Option<String>,
    pub settings_controller: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateEndowmentMsg {
    pub owner: String, // address that originally setup the endowment account
    pub maturity_time: Option<u64>, // datetime int of endowment maturity
    pub name: String,  // name of the Endowment
    pub categories: Categories, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub tier: Option<u8>, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub cw4_members: Vec<Member>,
    pub kyc_donors_only: bool,
    pub cw3_threshold: Threshold,
    pub cw3_max_voting_period: u64,
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub split_max: Decimal,
    pub split_min: Decimal,
    pub split_default: Decimal,
    pub earnings_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
    pub dao: Option<DaoSetup>,      // SubDAO setup options
    pub proposal_link: Option<u64>, // link back to the proposal that created an Endowment (set @ init)
    pub settings_controller: Option<SettingsController>,
    pub parent: Option<u64>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_splits: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentStatusMsg {
    pub endowment_id: u32,
    pub status: u8,
    pub beneficiary: Option<Beneficiary>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Strategy {
    pub vault: String,       // Vault SC Address
    pub percentage: Decimal, // percentage of funds to invest
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateEndowmentSettingsMsg {
    pub id: u32,
    pub owner: Option<String>,
    pub maturity_time: Option<Option<u64>>, // datetime int of endowment maturity
    pub strategies: Option<Vec<StrategyComponent>>, // list of vaults and percentage for locked/liquid accounts
    pub locked_endowment_configs: Option<Vec<String>>, // list of endowment configs that cannot be changed/altered once set at creation
    pub rebalance: Option<RebalanceDetails>,
    pub kyc_donors_only: Option<bool>,
    pub endow_type: Option<String>,
    pub name: Option<String>,
    pub categories: Option<Categories>,
    pub tier: Option<u8>,
    pub logo: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Tokens are sent back to an Account from a Vault
    VaultReceipt {
        id: u32,
        acct_type: AccountType,
    },
    // Tokens are sent back to an Account from a Swap
    SwapReceipt {
        id: u32,
        final_asset: Asset,
        acct_type: AccountType,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub id: u32,
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get all Config details for the contract
    Config {},
    // Get the balance of available UST and the invested portion balances
    Balance {
        id: u32,
    },
    // Get state details (like total donations received so far)
    State {
        id: u32,
    },
    // Get all Endowment details
    Endowment {
        id: u32,
    },
    // Gets list of all registered Endowments
    EndowmentList {
        proposal_link: Option<u64>,
        start_after: Option<u32>,
        limit: Option<u64>,
    },
    // Get endowment token balance
    TokenAmount {
        id: u32,
        asset_info: AssetInfo,
        acct_type: AccountType,
    },
    Allowances {
        id: u32,
        spender: String,
    },
}
