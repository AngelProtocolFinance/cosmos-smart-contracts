use angel_core::structs::{
    AcceptedTokens, AccountStrategies, BalanceInfo, DonationsReceived, EndowmentFee,
    EndowmentStatus, OneOffVaults, Profile, RebalanceDetails, SettingsController,
    StrategyComponent,
};
use cosmwasm_std::{Addr, Env, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub deposit_approved: bool, // DANO has approved to receive donations & transact
    pub withdraw_approved: bool, // DANO has approved to withdraw funds
    pub settings_controller: SettingsController,
    pub accepted_tokens: AcceptedTokens,
    pub next_account_id: u32,
    pub max_general_category_id: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr, // address that originally setup the endowment account
    pub status: EndowmentStatus,
    pub dao: Option<Addr>,           // subdao governance contract address
    pub dao_token: Option<Addr>,     // dao gov token contract address
    pub donation_match_active: bool, // donation matching contract address (None set for Charity Endowments as they just phone home to Registrar to get the addr)
    pub donation_match_contract: Option<Addr>, // contract for donation matching
    pub whitelisted_beneficiaries: Vec<String>, // if populated, only the listed Addresses can withdraw/receive funds from the Endowment (if empty, anyone can receive)
    pub whitelisted_contributors: Vec<String>, // if populated, only the listed Addresses can contribute to the Endowment (if empty, anyone can donate)
    pub maturity_time: Option<u64>,            // datetime int of endowment maturity (unit: seconds)
    pub deposit_approved: bool,                // approved to receive donations & transact
    pub withdraw_approved: bool,               // approved to withdraw funds
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub strategies: AccountStrategies, // vaults and percentages for locked/liquid accounts donations where auto_invest == TRUE
    pub oneoff_vaults: OneOffVaults, // vaults not covered in account startegies (more efficient tracking of vaults vs. looking up allll vaults)
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub earnings_fee: Option<EndowmentFee>, // Earnings Fee
    pub withdraw_fee: Option<EndowmentFee>, // Withdraw Fee
    pub deposit_fee: Option<EndowmentFee>, // Deposit Fee
    pub aum_fee: Option<EndowmentFee>, // AUM(Assets Under Management) Fee
    pub parent: Option<Addr>,        // Address of the Parent Endowment contract
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
    pub maturity_whitelist: Vec<Addr>, // list of addresses, which can withdraw after maturity date is reached (if any)
    pub profile: Profile,
    pub pending_redemptions: u8, // number of vault redemptions currently pending for this endowment
    pub copycat_strategy: Option<u32>, // endowment ID to copy their strategy
}

impl Endowment {
    pub fn is_expired(&self, env: &Env) -> bool {
        if let Some(maturity_time) = self.maturity_time {
            if env.block.time > Timestamp::from_seconds(maturity_time) {
                return true;
            }
        }
        false
    }
}

pub fn read_endowments(storage: &dyn Storage) -> StdResult<Vec<(u32, Endowment)>> {
    ENDOWMENTS
        .range(storage, None, None, Order::Ascending)
        .map(|item| {
            let (i, e) = item?;
            Ok((i, e))
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub donations_received: DonationsReceived,
    pub balances: BalanceInfo,
    pub closing_endowment: bool,
    pub closing_beneficiary: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATES: Map<u32, State> = Map::new("states");
pub const ENDOWMENTS: Map<u32, Endowment> = Map::new("endowments");
pub const COPYCATS: Map<u32, Vec<u32>> = Map::new("copycats");
