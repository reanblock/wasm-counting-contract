use cosmwasm_std::{
    to_json_binary, 
    Binary, 
    Deps,
    DepsMut, 
    Env, 
    MessageInfo, 
    Response, 
    StdResult,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use msg::InstantiateMsg;
use error::ContractError;

mod contract;
mod error;
mod state;
pub mod msg;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;
 
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg.counter, msg.minimal_donation)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;
 
    match msg {
        Donate {} => exec::donate(deps, info).map_err(ContractError::Std),
        Reset { counter } => exec::reset(deps, info, counter),
        Withdraw {} => exec::withdraw(deps, env, info),
        WithdrawTo { receiver, funds } => {
            exec::withdraw_to(deps, env, info, receiver, funds)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;
 
    match msg {
        Value {} => to_json_binary(&query::value(deps)?),
    }
}
