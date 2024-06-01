use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

mod contract;
pub mod msg;
 
#[entry_point]
pub fn instantiate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty,
) -> StdResult<Response> {
	Ok(Response::new())
}

#[entry_point]
pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;
 
    match msg {
        Value {} => to_json_binary(&query::value()),
        Incremented { value } => to_json_binary(&query::incremented(value)),
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result: usize = add(2, 2);
        assert_eq!(result, 4);
    }
}
