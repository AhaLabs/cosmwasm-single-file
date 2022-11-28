use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::entry_point;
use cosmwasm_std::StdError;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i8,
}

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i8,
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetNum returns the current count as a json-encoded number
    #[returns(GetNumResponse)]
    GetNum {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetNumResponse {
    pub count: i8,
}

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State { count: msg.count };
    STATE.save(deps.storage, &state)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => {
            STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                state.count += 1;
                Ok(state)
            })?;

            Ok(Response::new())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetNum {} => {
            let state = STATE.load(deps.storage)?;
            let count = GetNumResponse { count: state.count };
            to_binary(&count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetNum {}).unwrap();
        let value: GetNumResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetNum {}).unwrap();
        let value: GetNumResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }
}
