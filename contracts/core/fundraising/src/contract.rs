use crate::error::ContractError;
use crate::msg::{
    CreateMsg, DetailsResponse, ExecuteMsg, InstantiateMsg, ListResponse, QueryMsg, ReceiveMsg,
};
use crate::state::{
    all_campaign_ids, Campaign, Config, ContributorInfo, CAMPAIGNS, CONFIG, CONTRIBUTORS,
};
use angel_core::messages::registrar::QueryMsg::Config as RegistrarConfig;
use angel_core::responses::registrar::{
    ConfigResponse as RegistrarConfigResponse, EndowmentDetailResponse,
};
use angel_core::structs::{EndowmentStatus, GenericBalance};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

// version info for migration info
const CONTRACT_NAME: &str = "fundraising";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            registrar_contract: deps.api.addr_validate(&msg.registrar_contract)?,
            next_id: 1,
            campaign_max_days: msg.campaign_max_days,
            tax_rate: msg.tax_rate,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Create(msg) => {
            execute_create(deps, msg, Balance::from(info.funds), &info.sender)
        }
        ExecuteMsg::CloseCampaign { id } => execute_close_campaign(deps, env, info, id),
        ExecuteMsg::TopUp { id } => {
            execute_top_up(deps, &info.sender, id, Balance::from(info.funds))
        }
        ExecuteMsg::Contribute { id } => {
            execute_contribute(deps, &info.sender, id, Balance::from(info.funds))
        }
        ExecuteMsg::UpdateConfig {
            campaign_max_days,
            tax_rate,
        } => execute_update_config(deps, info, campaign_max_days, tax_rate),
        ExecuteMsg::Receive(msg) => execute_receive(deps, info, msg),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender.clone(),
        amount: wrapper.amount,
    });
    let api = deps.api;
    let sender = &api.addr_validate(&wrapper.sender)?;
    match msg {
        ReceiveMsg::Create(msg) => execute_create(deps, msg, balance, sender),
        ReceiveMsg::TopUp { id } => execute_top_up(deps, sender, id, balance),
        ReceiveMsg::Contribute { id } => execute_contribute(deps, sender, id, balance),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    campaign_max_days: u8,
    tax_rate: Decimal,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // only the Registrar Contract owner can update the configs
    if info.sender != registrar_config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // and save
    config.campaign_max_days = campaign_max_days;
    config.tax_rate = tax_rate;
    CONFIG.save(deps.storage, &config)?;

    let res = Response::new().add_attribute("action", "update_config");
    Ok(res)
}

pub fn execute_create(
    deps: DepsMut,
    msg: CreateMsg,
    balance: Balance,
    sender: &Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    // check the sender is an approved endowment in the Registrar
    let config = CONFIG.load(deps.storage)?;
    let endow_detail: EndowmentDetailResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.clone(),
        &angel_core::messages::registrar::QueryMsg::Endowment {
            endowment_addr: sender.to_string(),
        },
    )?;
    if endow_detail.endowment.status != EndowmentStatus::Approved {
        return Err(ContractError::Unauthorized {});
    }

    let locked_balance = match balance {
        Balance::Native(balance) => GenericBalance {
            native: balance.0,
            cw20: vec![],
        },
        Balance::Cw20(token) => GenericBalance {
            native: vec![],
            cw20: vec![token],
        },
    };

    let campaign = Campaign {
        open: true,
        creator: sender.clone(),
        title: msg.title,
        description: msg.description,
        image_url: msg.image_url,
        end_time: msg.end_time,
        funding_goal: msg.funding_goal,
        locked_balance,
        contributed_balance: GenericBalance::default(),
        contributors: vec![],
    };

    // try to store it, fail if the id was already in use
    let mut config = CONFIG.load(deps.storage)?;
    CAMPAIGNS.update(deps.storage, config.next_id, |existing| match existing {
        None => Ok(campaign),
        Some(_) => Err(ContractError::AlreadyInUse {}),
    })?;

    // increment the config next_id
    config.next_id += 1;
    CONFIG.save(deps.storage, &config)?;

    let res = Response::new().add_attributes(vec![
        ("action", "create"),
        ("id", &(config.next_id - 1).to_string()),
    ]);
    Ok(res)
}

pub fn execute_top_up(
    deps: DepsMut,
    sender: &Addr,
    id: u64,
    balance: Balance,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // this fails is no campaign there
    let mut campaign = CAMPAIGNS.load(deps.storage, id)?;

    // top-ups can only be done while the campaign is open
    if !campaign.open {
        return Err(ContractError::Expired {});
    }

    // only the campaign creator can top-up their campaign's locked balance
    if sender != &campaign.creator {
        return Err(ContractError::Unauthorized {});
    }

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the whitelist
        if !campaign
            .funding_goal
            .cw20
            .iter()
            .any(|t| &t.address == &token.address)
        {
            return Err(ContractError::NotInWhitelist {});
        }
    };

    campaign.locked_balance.add_tokens(balance);

    // and save
    CAMPAIGNS.save(deps.storage, id, &campaign)?;

    let res = Response::new().add_attributes(vec![("action", "top_up"), ("id", &id.to_string())]);
    Ok(res)
}

pub fn execute_contribute(
    deps: DepsMut,
    sender: &Addr,
    id: u64,
    balance: Balance,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // this fails is no campaign there
    let mut campaign = CAMPAIGNS.load(deps.storage, id)?;

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the whitelist
        if !campaign
            .funding_goal
            .cw20
            .iter()
            .any(|t| &t.address == &token.address)
        {
            return Err(ContractError::NotInWhitelist {});
        }
    };

    // get the contributor's balance for the given campaign and credit them
    let contributions: Vec<ContributorInfo> = match CONTRIBUTORS.may_load(deps.storage, sender)? {
        Some(mut contributor) => {
            let pos = contributor
                .iter()
                .position(|camp_bal| &camp_bal.campaign == &id);

            if pos != None {
                contributor[pos.unwrap()]
                    .balance
                    .add_tokens(balance.clone());
            } else {
                let mut default_bal = GenericBalance::default();
                default_bal.add_tokens(balance.clone());
                contributor.push(ContributorInfo {
                    campaign: id,
                    balance: default_bal,
                    rewards_claimed: false,
                })
            }
            contributor
        }
        None => {
            let mut default_bal = GenericBalance::default();
            default_bal.add_tokens(balance.clone());
            vec![ContributorInfo {
                campaign: id,
                balance: default_bal,
                rewards_claimed: false,
            }]
        }
    };

    // update the campaign's generic "total" contributions balance as well
    campaign.contributed_balance.add_tokens(balance);

    // and save
    CAMPAIGNS.save(deps.storage, id, &campaign)?;
    CONTRIBUTORS.save(deps.storage, sender, &contributions)?;

    let res =
        Response::new().add_attributes(vec![("action", "contribute"), ("id", &id.to_string())]);
    Ok(res)
}

pub fn execute_close_campaign(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // this fails if no campaign there
    let mut campaign = CAMPAIGNS.load(deps.storage, id)?;
    let config = CONFIG.load(deps.storage)?;

    if !campaign.is_expired(&env) {
        return Err(ContractError::NotExpired {});
    }

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarConfig {})?,
        }))?;

    // calculate the amount of withholding tax due to the AP Treasury
    let (balance_less_tax, withholding_balance) =
        calculate_witholding(config.tax_rate, &campaign.contributed_balance);
    // send all contributed tokens, less tax, to the campaign's creator
    let contributor_messages: Vec<SubMsg> = send_tokens(&campaign.creator, &balance_less_tax)?;
    let treasury_messages: Vec<SubMsg> = send_tokens(
        &deps.api.addr_validate(&registrar_config.treasury)?,
        &withholding_balance,
    )?;

    // disable the campaign and it's contributions data
    campaign.open = false;
    CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new()
        .add_attribute("action", "close-campaign")
        .add_attribute("id", &id.to_string())
        // sends funds raised (less tax) to the creator
        .add_submessages(contributor_messages)
        // sends the taxes due on funds raised to AP Treasury
        .add_submessages(treasury_messages))
}

pub fn execute_claim_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // try to look up given campaign. Will fail if not found.
    let campaign = CAMPAIGNS.load(deps.storage, id)?;

    // check that the campaign is closed before sending rewards
    if campaign.open {
        return Err(ContractError::CampaignIsOpen {});
    }

    let mut contributor = CONTRIBUTORS.load(deps.storage, &info.sender)?;
    // check that the msg sender made contributions to given campaign
    let pos = contributor
        .iter()
        .position(|camp_bal| &camp_bal.campaign == &id);
    if pos == None {
        return Err(ContractError::InvalidInputs {});
    }
    // now we have record of everything the sender contributed to this campaign & status
    let mut camp_contrib = contributor[pos.unwrap()].clone();

    // check the user has not already claimed their rewards
    if camp_contrib.rewards_claimed {
        return Err(ContractError::CannotClaimRewards {});
    }

    // TO DO: FINALIZE THE LOGIC TO PAY OUT REWARD CLAIMS!!
    // 1. Calculate the amount of rewards a user should get: (their contrib / total contrib) * locked_balance
    // 2. process reward claims to user using the `send_tokens()` w/ mod'd Locked Bal amnt

    // 3. mark rewards as claimed for this user
    camp_contrib.rewards_claimed = true;
    contributor[pos.unwrap()] = camp_contrib;
    CONTRIBUTORS.save(deps.storage, &info.sender, &contributor)?;

    Ok(Response::default())
}

/// Args: current tax rate and contributed balance as args
/// Return: a tuple of two generic balances:
/// (ContributedBalanceLessTax, WithholdingTaxAmount)
fn calculate_witholding(
    tax_rate: Decimal,
    balance: &GenericBalance,
) -> (GenericBalance, GenericBalance) {
    let mut contributed_less_tax = balance.clone();
    let mut withholding_balance = GenericBalance::default();

    for coin in balance.native.iter() {
        // add tax witholding
        withholding_balance.add_tokens(Balance::from(vec![Coin {
            amount: coin.amount * tax_rate,
            denom: coin.denom.clone(),
        }]));
        // deduct tax witheld from contributed native balance
        contributed_less_tax.deduct_tokens(Balance::from(vec![Coin {
            amount: coin.amount * tax_rate,
            denom: coin.denom.clone(),
        }]));
    }

    for token in balance.cw20.iter() {
        // add tax witholding
        withholding_balance.add_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: token.amount * tax_rate,
            address: token.address.clone(),
        }));
        // deduct tax witheld from contributed cw20 token balance
        contributed_less_tax.deduct_tokens(Balance::Cw20(Cw20CoinVerified {
            amount: token.amount * tax_rate,
            address: token.address.clone(),
        }));
    }
    // return the adjusted contributed balance & tax owed balances
    (contributed_less_tax, withholding_balance)
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<SubMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::List {} => to_binary(&query_list(deps)?),
        QueryMsg::Details { id } => to_binary(&query_details(deps, id)?),
    }
}

fn query_details(deps: Deps, id: u64) -> StdResult<DetailsResponse> {
    let campaign = CAMPAIGNS.load(deps.storage, id)?;
    let details = DetailsResponse {
        id,
        creator: campaign.creator.into(),
        title: campaign.title,
        description: campaign.description,
        image_url: campaign.image_url,
        end_time: campaign.end_time,
        funding_goal: campaign.funding_goal,
        locked_balance: campaign.locked_balance,
        contributor_count: campaign.contributors.len() as u64,
        contributed_balance: campaign.contributed_balance,
    };
    Ok(details)
}

fn query_list(deps: Deps) -> StdResult<ListResponse> {
    Ok(ListResponse {
        campaigns: all_campaign_ids(deps.storage)?,
    })
}
