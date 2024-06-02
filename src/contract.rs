use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult, Decimal};
use cw2::set_contract_version;
// use cw_storage_plus::Item;
// use serde::{Deserialize, Serialize};
// use crate::error::ContractError;
use crate::msg::Parent;
use crate::state::{ParentDonation, State, PARENT_DONATION, STATE};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    counter: u64,
    minimal_donation: Coin,
    parent: Option<Parent>,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
            owner: info.sender,
            donating_parent: parent.as_ref().map(|p| p.donating_period),
        },
    )?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            },
        )?;
    }

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::ValueResp;
    use crate::state::STATE;

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp { value })
    }
}

pub mod exec {
    use cosmwasm_std::{
        to_json_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg, Decimal
    };

    use crate::error::ContractError;
    use crate::msg::ExecMsg;
    use crate::state::{PARENT_DONATION, STATE};

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;

            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;

                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period;

                    let funds: Vec<_> = deps
                        .querier
                        .query_all_balances(env.contract.address)?
                        .into_iter()
                        .map(|mut coin| {
                            // TODO FIX  cannot multiply `Uint128` by `Decimal` !!
                            // coin.amount = coin.amount * parent_donation.part;
                            coin
                        })
                        .collect();

                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_json_binary(&ExecMsg::Donate {})?,
                        funds,
                    };

                    resp = resp
                        .add_message(msg)
                        .add_attribute("donated_to_parent", parent_donation.address.to_string());
                }
            }

            STATE.save(deps.storage, &state)?;
        }

        resp = resp
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());

        Ok(resp)
    }

    pub fn reset(
        deps: DepsMut,
        info: MessageInfo,
        counter: u64,
    ) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;

        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {
                owner: state.owner.to_string(),
            });
        }

        state.counter = counter;
        STATE.save(deps.storage, &state)?;

        let resp = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        receiver: String,
        funds: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let mut balance = deps.querier.query_all_balances(&env.contract.address)?;

        if !funds.is_empty() {
            for coin in &mut balance {
                let limit = funds
                    .iter()
                    .find(|c| c.denom == coin.denom)
                    .map(|c| c.amount)
                    .unwrap_or(Uint128::zero());

                coin.amount = std::cmp::min(coin.amount, limit);
            }
        }

        let bank_msg = BankMsg::Send {
            to_address: receiver,
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}