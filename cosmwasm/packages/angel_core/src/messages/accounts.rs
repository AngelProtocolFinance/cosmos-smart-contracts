use crate::responses::accounts::*;
use crate::structs::{
    AccountType, Allowances, Beneficiary, Categories, DaoSetup, EndowmentController, EndowmentFee,
    EndowmentType, RebalanceDetails, SplitDetails, StrategyInvestment, SwapOperation,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use cw4::Member;
use cw_asset::{Asset, AssetUnchecked};
use cw_utils::{Expiration, Threshold};

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner_sc: String,
    pub registrar_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    // Add tokens sent for a specific account
    Deposit(DepositMsg),
    // Pull funds from Endowment locked/liquid free balances (TOH) to a Beneficiary address or an AIF
    Withdraw {
        id: u32,
        acct_type: AccountType,
        beneficiary_wallet: Option<String>,
        beneficiary_endow: Option<u32>,
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
    // Invest TOH funds into Strategies
    StrategiesInvest {
        id: u32,
        strategies: Vec<StrategyInvestment>,
    },
    // Redeem TOH funds from Strategies
    StrategiesRedeem {
        id: u32,
        strategies: Vec<StrategyInvestment>,
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
    // Allows the SC owner (only!) to change ownership & upper limit of general categories ID allowed
    UpdateConfig {
        new_owner: Option<String>,
        new_registrar: Option<String>,
        max_general_category_id: Option<u8>,
        ibc_controller: Option<String>,
    },
    // Update an Endowment owner, beneficiary, and other core items
    UpdateEndowmentDetails(UpdateEndowmentDetailsMsg),
    // Update an Endowment ability to receive/send funds
    UpdateEndowmentStatus(UpdateEndowmentStatusMsg),
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

#[cw_serde]
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
    pub beneficiaries_allowlist: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can)
    pub contributors_allowlist: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub split_max: Decimal,
    pub split_min: Decimal,
    pub split_default: Decimal,
    pub earnings_fee: Option<EndowmentFee>,
    pub withdraw_fee: Option<EndowmentFee>,
    pub deposit_fee: Option<EndowmentFee>,
    pub aum_fee: Option<EndowmentFee>,
    pub dao: Option<DaoSetup>,      // SubDAO setup options
    pub proposal_link: Option<u64>, // link back to the proposal that created an Endowment (set @ init)
    pub endowment_controller: Option<EndowmentController>,
    pub parent: Option<u32>,
    pub split_to_liquid: Option<SplitDetails>,
    pub ignore_user_splits: bool,
    pub referral_id: Option<u32>,
}

#[cw_serde]
pub struct UpdateEndowmentStatusMsg {
    pub endowment_id: u32,
    pub status: u8,
    pub beneficiary: Option<Beneficiary>,
}

#[cw_serde]
pub struct UpdateEndowmentDetailsMsg {
    pub id: u32,
    pub owner: Option<String>,
    pub rebalance: Option<RebalanceDetails>,
    pub kyc_donors_only: Option<bool>,
    pub endow_type: Option<String>,
    pub name: Option<String>,
    pub categories: Option<Categories>,
    pub tier: Option<u8>,
    pub logo: Option<String>,
    pub image: Option<String>,
}

#[cw_serde]
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

#[cw_serde]
pub struct DepositMsg {
    pub id: u32,
    pub locked_percentage: Decimal,
    pub liquid_percentage: Decimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Get all Config details for the contract
    #[returns(ConfigResponse)]
    Config {},
    // Get state details (like tokens on hand balances, total donations received, etc)
    #[returns(StateResponse)]
    State { id: u32 },
    // Get all Endowment details
    #[returns(EndowmentDetailsResponse)]
    Endowment { id: u32 },
    // Gets the Endowment by "proposal_link"
    #[returns(EndowmentDetailsResponse)]
    EndowmentByProposalLink { proposal_link: u64 },
    // Get the Allowances for Endowment
    #[returns(Allowances)]
    Allowances { id: u32, spender: String },
}
