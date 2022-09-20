use crate::error::ContractError;
use crate::msg::{
    CreateMsg, DetailsResponse, ExecuteMsg, InstantiateMsg, ListResponse, QueryMsg, ReceiveMsg,
};
use crate::state::{
    all_campaigns, Campaign, Config, ContributorInfo, CAMPAIGNS, CONFIG, CONTRIBUTORS,
};
use angel_core::messages::accounts::QueryMsg as AccountQueryMsg;
use angel_core::messages::registrar::QueryMsg as RegistrarQueryMsg;
use angel_core::responses::accounts::EndowmentDetailsResponse;
use angel_core::responses::registrar::ConfigResponse as RegistrarConfigResponse;
use angel_core::structs::{EndowmentStatus, GenericBalance};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg, WasmQuery,
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
            campaign_period_seconds: msg.campaign_period_seconds,
            tax_rate: msg.tax_rate,
            accepted_tokens: msg.accepted_tokens,
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
        ExecuteMsg::Create { endowment_id, msg } => execute_create(
            deps,
            env,
            endowment_id,
            msg,
            Balance::from(info.funds),
            &info.sender,
        ),
        ExecuteMsg::CloseCampaign { id } => execute_close_campaign(deps, env, info, id),
        ExecuteMsg::TopUp { id } => {
            execute_top_up(deps, env, &info.sender, id, Balance::from(info.funds))
        }
        ExecuteMsg::Contribute { id } => {
            execute_contribute(deps, env, &info.sender, id, Balance::from(info.funds))
        }
        ExecuteMsg::UpdateConfig {
            campaign_period_seconds,
            tax_rate,
            accepted_tokens,
        } => execute_update_config(
            deps,
            info,
            campaign_period_seconds,
            tax_rate,
            accepted_tokens,
        ),
        ExecuteMsg::ClaimRewards { id } => execute_claim_rewards(deps, env, info, id),
        ExecuteMsg::RefundContributions { id } => execute_refund_contributions(deps, env, info, id),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender,
        amount: wrapper.amount,
    });
    let api = deps.api;
    let sender = &api.addr_validate(&wrapper.sender)?;
    match msg {
        ReceiveMsg::Create { endowment_id, msg } => {
            execute_create(deps, env, endowment_id, msg, balance, sender)
        }
        ReceiveMsg::TopUp { id } => execute_top_up(deps, env, sender, id, balance),
        ReceiveMsg::Contribute { id } => execute_contribute(deps, env, sender, id, balance),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    campaign_period_seconds: u64,
    tax_rate: Decimal,
    accepted_tokens: GenericBalance,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let registrar_config: RegistrarConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.registrar_contract.to_string(),
            msg: to_binary(&RegistrarQueryMsg::Config {}).unwrap(),
        }))?;

    // only the Registrar Contract owner can update the configs
    if info.sender != registrar_config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // and save
    config.campaign_period_seconds = campaign_period_seconds;
    config.tax_rate = tax_rate;
    config.accepted_tokens = accepted_tokens;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_create(
    deps: DepsMut,
    env: Env,
    endowment_id: u32,
    msg: CreateMsg,
    balance: Balance,
    sender: &Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty()
        || (msg.funding_goal.native.is_empty() && msg.funding_goal.cw20.is_empty())
    {
        return Err(ContractError::EmptyBalance {});
    }

    // check the sender is an approved endowment in the Registrar
    let config = CONFIG.load(deps.storage)?;
    let accounts_contract = sender.to_string();
    let registrar_config: RegistrarConfigResponse = deps.querier.query_wasm_smart(
        config.registrar_contract.clone(),
        &RegistrarQueryMsg::Config {},
    )?;

    match registrar_config.accounts_contract {
        Some(addr) => {
            if addr != accounts_contract {
                return Err(ContractError::Unauthorized {});
            }
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "accounts_contract not exist".to_string(),
            }))
        }
    }

    let endow_detail: EndowmentDetailsResponse = deps.querier.query_wasm_smart(
        accounts_contract,
        &AccountQueryMsg::Endowment { id: endowment_id },
    )?;

    if endow_detail.status != EndowmentStatus::Approved {
        return Err(ContractError::Unauthorized {});
    }

    // check that the user supplied campaign end time less
    // the max campaign length is less than or equal to the
    // current block time
    if Timestamp::from_seconds(msg.end_time_epoch - config.campaign_period_seconds)
        <= env.block.time
    {
        return Err(ContractError::InvalidInputs {});
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

    let mut contributed_balance = GenericBalance::default();
    let mut funding_threshold = GenericBalance::default();
    // check we have a single token to use as the desired contribution token for the campaign
    if msg.funding_goal.native.iter().len() == 1 && msg.funding_goal.cw20.iter().len() == 0 {
        for coin in msg.funding_goal.native.iter() {
            let pos = config
                .accepted_tokens
                .native
                .iter()
                .position(|c| c.denom == coin.denom);

            if pos.is_none() {
                return Err(ContractError::NotInWhitelist {
                    token_type: "native".to_string(),
                    given: coin.denom.clone(),
                });
            }

            funding_threshold.set_token_balances(Balance::from(vec![Coin {
                amount: coin.amount * msg.reward_threshold,
                denom: coin.denom.clone(),
            }]));

            contributed_balance.set_token_balances(Balance::from(vec![Coin {
                amount: Uint128::zero(),
                denom: coin.denom.clone(),
            }]));
        }
    } else if msg.funding_goal.cw20.iter().len() == 1 && msg.funding_goal.native.iter().len() == 0 {
        for token in msg.funding_goal.cw20.iter() {
            let pos = config
                .accepted_tokens
                .cw20
                .iter()
                .position(|t| t.address == token.address);

            if pos.is_none() {
                return Err(ContractError::NotInWhitelist {
                    token_type: "CW20".to_string(),
                    given: token.address.to_string(),
                });
            }

            funding_threshold.set_token_balances(Balance::Cw20(Cw20CoinVerified {
                amount: token.amount * msg.reward_threshold,
                address: token.address.clone(),
            }));

            contributed_balance.set_token_balances(Balance::Cw20(Cw20CoinVerified {
                amount: Uint128::zero(),
                address: token.address.clone(),
            }));
        }
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: "Contract only accepts a single CW20 or Native Token at a time for funding goal"
                .to_string(),
        }));
    }

    let campaign = Campaign {
        open: true,
        success: false,
        creator: sender.clone(),
        title: msg.title,
        description: msg.description,
        image_url: msg.image_url,
        end_time_epoch: msg.end_time_epoch,
        funding_goal: msg.funding_goal,
        funding_threshold,
        locked_balance,
        contributed_balance,
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

    Ok(Response::new().add_attributes(vec![
        ("action", "create"),
        ("id", &(config.next_id - 1).to_string()),
    ]))
}

pub fn execute_top_up(
    deps: DepsMut,
    env: Env,
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
    if !campaign.open || campaign.is_expired(&env) {
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
            .any(|t| t.address == token.address)
        {
            return Err(ContractError::NotInWhitelist {
                token_type: "CW20".to_string(),
                given: "tokens".to_string(),
            });
        }
    };

    campaign.locked_balance.add_tokens(balance);

    // and save
    CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attributes(vec![("action", "top_up"), ("id", &id.to_string())]))
}

pub fn execute_contribute(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    id: u64,
    balance: Balance,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // this fails is no campaign there
    let mut campaign = CAMPAIGNS.load(deps.storage, id)?;
    if !campaign.open || campaign.is_expired(&env) {
        return Err(ContractError::Expired {});
    }

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the whitelist
        if !campaign
            .funding_goal
            .cw20
            .iter()
            .any(|t| t.address == token.address)
        {
            return Err(ContractError::NotInWhitelist {
                token_type: "CW20".to_string(),
                given: "tokens".to_string(),
            });
        }
    };

    // get the contributor's balance for the given campaign and credit them
    let contributions: Vec<ContributorInfo> = match CONTRIBUTORS.may_load(deps.storage, sender)? {
        Some(mut contributor) => {
            let pos = contributor
                .iter()
                .position(|camp_bal| camp_bal.campaign == id);

            if let Some(pos) = pos {
                contributor[pos].balance.add_tokens(balance.clone());
            } else {
                let mut default_bal = GenericBalance::default();
                default_bal.add_tokens(balance.clone());
                contributor.push(ContributorInfo {
                    campaign: id,
                    balance: default_bal,
                    rewards_claimed: false,
                    contribution_refunded: false,
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
                contribution_refunded: false,
            }]
        }
    };

    // update the campaign's generic "total" contributions balance as well
    campaign.contributed_balance.add_tokens(balance);
    // make sure the contributor's addr is noted for this campaign
    if !campaign.contributors.iter().any(|addr| addr == sender) {
        campaign.contributors.push(sender.clone())
    }

    // and save
    CAMPAIGNS.save(deps.storage, id, &campaign)?;
    CONTRIBUTORS.save(deps.storage, sender, &contributions)?;

    Ok(Response::new().add_attributes(vec![("action", "contribute"), ("id", &id.to_string())]))
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
            msg: to_binary(&RegistrarQueryMsg::Config {})?,
        }))?;

    let contributor_messages: Vec<SubMsg>;
    let mut treasury_messages: Vec<SubMsg> = vec![];
    // did the campaign meet the threshold funding to release rewards / funds?
    if threshold_met(
        campaign.contributed_balance.clone(),
        campaign.funding_threshold.clone(),
    ) {
        campaign.success = true;
        // calculate the amount of withholding tax due to the AP Treasury
        let (balance_less_tax, withholding_balance) =
            calculate_witholding(config.tax_rate, &campaign.contributed_balance);
        // send all contributed tokens, less tax, to the campaign's creator
        contributor_messages = send_tokens(&campaign.creator, &balance_less_tax)?;
        treasury_messages = send_tokens(
            &deps.api.addr_validate(&registrar_config.treasury)?,
            &withholding_balance,
        )?;
    } else {
        // Send all locked rewards back to the creator.
        // Users will now be able to get a refund on their contributions after closing.
        contributor_messages = send_tokens(&campaign.creator, &campaign.locked_balance)?;
    }

    // disable the campaign and it's contributions data
    campaign.open = false;
    CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new()
        .add_attribute("action", "close-campaign")
        .add_attribute("id", &id.to_string())
        .add_attribute("campaign_succeeded", campaign.success.to_string())
        // sends to creator: funds raised (less tax) if passed OR send rewards back if failed
        .add_submessages(contributor_messages)
        // sends to AP Treasury: taxes due on funds raised
        .add_submessages(treasury_messages))
}

pub fn execute_refund_contributions(
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
        .position(|camp_bal| camp_bal.campaign == id);
    if pos.is_none() {
        return Err(ContractError::InvalidInputs {});
    }
    // now we have record of everything the sender contributed to this campaign & status
    let mut camp_contrib = contributor[pos.unwrap()].clone();

    // check the user has not already been refunded
    if camp_contrib.contribution_refunded {
        return Err(ContractError::AlreadyRefunded {});
    }

    // mark contributions as refunded for this user
    camp_contrib.contribution_refunded = true;
    contributor[pos.unwrap()] = camp_contrib.clone();
    CONTRIBUTORS.save(deps.storage, &info.sender, &contributor)?;

    Ok(Response::default()
        .add_submessages(send_tokens(&info.sender, &camp_contrib.balance)?)
        .add_attribute("action", "refund_contributions")
        .add_attribute("id", &id.to_string()))
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
        .position(|camp_bal| camp_bal.campaign == id);
    if pos.is_none() {
        return Err(ContractError::InvalidInputs {});
    }
    // now we have record of everything the sender contributed to this campaign & status
    let mut camp_contrib = contributor[pos.unwrap()].clone();

    // check the user has not already claimed their rewards
    if camp_contrib.rewards_claimed {
        return Err(ContractError::CannotClaimRewards {});
    }

    let mut rewards = GenericBalance::default();
    // 1. Calculate the amount of rewards a user can claim: (user contrib / total contrib) * locked_balance
    for coin in campaign.contributed_balance.native.iter() {
        let user_amnt = campaign
            .contributed_balance
            .get_denom_amount(coin.denom.to_string())
            .amount
            .checked_div(coin.amount)
            .unwrap();
        if user_amnt > Uint128::zero() {
            // 2. Push valid reward claims into balance to be used w. send_tokens()
            rewards.set_token_balances(Balance::from(vec![Coin {
                amount: user_amnt
                    * campaign
                        .locked_balance
                        .get_denom_amount(coin.denom.clone())
                        .amount,
                denom: coin.denom.clone(),
            }]));
        }
    }
    for token in campaign.contributed_balance.cw20.iter() {
        let user_amnt = campaign
            .contributed_balance
            .get_token_amount(token.address.clone())
            .amount
            .checked_div(token.amount)
            .unwrap();
        if user_amnt > Uint128::zero() {
            // 2. Push valid reward claims into balance to be used w. send_tokens()
            rewards.set_token_balances(Balance::Cw20(Cw20CoinVerified {
                amount: user_amnt
                    * campaign
                        .locked_balance
                        .get_token_amount(token.address.clone())
                        .amount,
                address: token.address.clone(),
            }));
        }
    }

    // 3. mark rewards as claimed for this user
    camp_contrib.rewards_claimed = true;
    contributor[pos.unwrap()] = camp_contrib;
    CONTRIBUTORS.save(deps.storage, &info.sender, &contributor)?;

    Ok(Response::default()
        // 4. Append submessages to transfer all rewards due to user
        .add_submessages(send_tokens(&info.sender, &rewards)?)
        .add_attribute("action", "claim_rewards")
        .add_attribute("id", &id.to_string()))
}

/// Threshold will have a single CW20 or single native token to check against the
/// Contributed Balance (also a single CW20 or Native token)
fn threshold_met(contributed: GenericBalance, threshold: GenericBalance) -> bool {
    let mut result = false;
    for coin in threshold.native.iter() {
        let index = contributed
            .native
            .iter()
            .enumerate()
            .find_map(|(i, exist)| {
                if exist.denom == coin.denom {
                    Some(i)
                } else {
                    None
                }
            });
        result = match index {
            None => false,
            Some(idx) => contributed.native[idx].amount >= coin.amount,
        }
    }
    for token in threshold.cw20.iter() {
        let index = contributed.cw20.iter().enumerate().find_map(|(i, exist)| {
            if exist.address == token.address {
                Some(i)
            } else {
                None
            }
        });
        result = match index {
            None => false,
            Some(idx) => contributed.cw20[idx].amount >= token.amount,
        }
    }
    result
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
        QueryMsg::List {
            creator,
            open,
            success,
        } => to_binary(&query_list(deps, creator, open, success)?),
        QueryMsg::Details { id } => to_binary(&query_details(deps, id)?),
        QueryMsg::ContributorCampaigns { contributor } => {
            to_binary(&query_list_by_contributor(deps, contributor)?)
        }
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
        end_time_epoch: campaign.end_time_epoch,
        funding_goal: campaign.funding_goal,
        funding_threshold: campaign.funding_threshold,
        locked_balance: campaign.locked_balance,
        contributor_count: campaign.contributors.len() as u64,
        contributed_balance: campaign.contributed_balance,
    };
    Ok(details)
}

fn query_list(
    deps: Deps,
    creator: Option<Addr>,
    open: Option<bool>,
    success: Option<bool>,
) -> StdResult<ListResponse> {
    let campaigns: Vec<Campaign> = all_campaigns(deps.storage)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.creator.as_ref() == creator.as_ref().unwrap_or(&c.creator).as_ref())
        .filter(|c| c.open == open.unwrap_or(c.open))
        .filter(|c| c.success == success.unwrap_or(c.success))
        .collect::<Vec<Campaign>>();

    Ok(ListResponse { campaigns })
}

fn query_list_by_contributor(deps: Deps, contributor: String) -> StdResult<ListResponse> {
    let contrib_addr = deps.api.addr_validate(&contributor)?;
    let campaigns: Vec<Campaign> = all_campaigns(deps.storage)
        .unwrap_or_default()
        .into_iter()
        .filter(|campaign| campaign.contributors.iter().any(|c| c == &contrib_addr))
        .collect::<Vec<Campaign>>();

    Ok(ListResponse { campaigns })
}
