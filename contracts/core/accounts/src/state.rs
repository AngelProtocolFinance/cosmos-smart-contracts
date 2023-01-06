use angel_core::structs::{
    AccountStrategies, BalanceInfo, Beneficiary, Categories, DonationsReceived, EndowmentStatus,
    EndowmentType, OneOffVaults, RebalanceDetails,
};
use cosmwasm_std::{Addr, Env, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: u64 = 15;
const MAX_LIMIT: u64 = 80;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr, // DANO/AP Team Address
    pub registrar_contract: Addr,
    pub ibc_controller: Addr, // created to allow IBC packet sending to other Cosmos chains
    pub next_account_id: u32,
    pub max_general_category_id: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct OldEndowment {
    pub owner: Addr,
    pub name: String,
    pub categories: Categories,
    pub tier: Option<u8>,
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub status: EndowmentStatus,
    pub deposit_approved: bool,
    pub withdraw_approved: bool,
    pub withdraw_before_maturity: bool,
    pub maturity_time: Option<u64>,
    pub maturity_height: Option<u64>,
    pub strategies: AccountStrategies,
    pub oneoff_vaults: OneOffVaults,
    pub rebalance: RebalanceDetails,
    pub kyc_donors_only: bool,
    pub pending_redemptions: u8,
    pub copycat_strategy: Option<u32>,
    pub proposal_link: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Endowment {
    pub owner: Addr,
    pub name: String,           // name of the Endowment
    pub categories: Categories, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub tier: Option<u8>, // SHOULD NOT be editable for now (only the Config.owner, ie via the Gov contract or AP CW3 Multisig can set/update)
    pub endow_type: EndowmentType,
    pub logo: Option<String>,
    pub image: Option<String>,
    pub status: EndowmentStatus,
    pub deposit_approved: bool, // approved to receive donations & transact
    pub withdraw_approved: bool, // approved to withdraw funds
    pub maturity_time: Option<u64>, // datetime int of endowment maturity
    pub strategies: AccountStrategies, // vaults and percentages for locked/liquid accounts donations where auto_invest == TRUE
    pub oneoff_vaults: OneOffVaults, // vaults not covered in account startegies (more efficient tracking of vaults vs. looking up allll vaults)
    pub rebalance: RebalanceDetails, // parameters to guide rebalancing & harvesting of gains from locked/liquid accounts
    pub kyc_donors_only: bool, // allow owner to state a preference for receiving only kyc'd donations (where possible)
    pub pending_redemptions: u8, // number of vault redemptions currently pending for this endowment
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

pub fn read_endowments(
    storage: &dyn Storage,
    proposal_link: Option<u64>,
    start_after: Option<u32>,
    limit: Option<u64>,
) -> StdResult<Vec<(u32, Endowment)>> {
    let start: Option<Bound<u32>> = match start_after {
        Some(id) => Some(Bound::exclusive(id)),
        None => None,
    };
    match proposal_link {
        Some(proposal_id) => ENDOWMENTS
            .range(storage, start, None, Order::Ascending)
            .filter(|e| e.as_ref().unwrap().1.proposal_link == Some(proposal_id))
            .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
            .map(|item| {
                let (i, e) = item?;
                Ok((i, e))
            })
            .collect(),
        None => ENDOWMENTS
            .range(storage, start, None, Order::Ascending)
            .take(limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize)
            .map(|item| {
                let (i, e) = item?;
                Ok((i, e))
            })
            .collect(),
    }
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
