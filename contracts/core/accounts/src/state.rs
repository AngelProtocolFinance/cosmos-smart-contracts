use angel_core::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, DonationsReceived, EndowmentStatus, OneOffVaults,
    Profile, RebalanceDetails,
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
    pub next_account_id: u32,
    pub max_general_category_id: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr,
    pub status: EndowmentStatus,
    pub deposit_approved: bool, // approved to receive donations & transact
    pub withdraw_approved: bool, // approved to withdraw funds
    pub withdraw_before_maturity: bool, // endowment allowed to withdraw funds from locked acct before maturity date
    pub maturity_time: Option<u64>,     // datetime int of endowment maturity
    pub maturity_height: Option<u64>,   // block equiv of the maturity_datetime
    pub strategies: AccountStrategies, // vaults and percentages for locked/liquid accounts donations where auto_invest == TRUE
    pub oneoff_vaults: OneOffVaults, // vaults not covered in account startegies (more efficient tracking of vaults vs. looking up allll vaults)
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
    pub profile: Profile,
    pub pending_redemptions: u8, // number of vault redemptions currently pending for this endowment
    pub copycat_strategy: Option<u32>, // endowment ID to copy their strategy
    pub proposal_link: Option<u64>, // link back the CW3 Proposal that created an endowment
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
    pub closing_beneficiary: Option<Beneficiary>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATES: Map<u32, State> = Map::new("states");
pub const ENDOWMENTS: Map<u32, Endowment> = Map::new("endowments");
pub const COPYCATS: Map<u32, Vec<u32>> = Map::new("copycats");
