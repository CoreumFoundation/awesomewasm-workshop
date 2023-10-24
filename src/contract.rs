use crate::msg::AmountResponse;

// coreum deps
use coreum_wasm_sdk::assetft;
use coreum_wasm_sdk::core::{CoreumMsg, CoreumQueries};

use cosmwasm_std::{entry_point, to_binary, Binary, Deps, QueryRequest, StdResult};
use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};
use thiserror::Error;
use crate::state::{STATE, State};

// version info for migration info
const CONTRACT_NAME: &str = "creates.io:ft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]

// here we define our message we will send to the contract to instantiate it
pub struct InstantiateMsg {
    pub symbol: String,
    pub subunit: String,
    pub precision: u32,
    pub initial_amount: Uint128,
    pub airdrop_amount: Uint128,
}

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput(String),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    MintForAirdrop { amount: u128 },
    ReceiveAirdrop {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    match msg {
        ExecuteMsg::MintForAirdrop { amount } => mint_for_airdrop(deps, info, amount),
        ExecuteMsg::ReceiveAirdrop {} => receive_airdrop(deps, info),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Token {},
    MintedForAirdrop {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<CoreumQueries>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Token {} => token(deps),
        QueryMsg::MintedForAirdrop {} => minted_for_airdrop(deps),
    }
}

// ********** Instantiate **********

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<CoreumMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let issue_msg = CoreumMsg::AssetFT(assetft::Msg::Issue {
        symbol: msg.symbol,
        subunit: msg.subunit.clone(),
        precision: msg.precision,
        initial_amount: msg.initial_amount,
        description: None,
        features: Some(vec![0]), // 0 - minting
        burn_rate: Some("0".into()),
        send_commission_rate: Some("0.1".into()), // 10% commission for sending
    });

    let denom = format!("{}-{}", msg.subunit, env.contract.address).to_lowercase();

    let state = State {
        owner: info.sender.into(),
        denom,
        minted_for_airdrop: msg.initial_amount,
        airdrop_amount: msg.airdrop_amount,
    };

    //store into state
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("owner", state.owner)
        .add_attribute("denom", state.denom)
        .add_message(issue_msg)
    )
}

// ********** Transactions **********

fn mint_for_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    amount: u128,
) -> Result<Response<CoreumMsg>, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    //simple access control
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let msg = CoreumMsg::AssetFT(assetft::Msg::Mint {
        coin: Coin::new(amount, state.denom.clone()),
    });

    state.minted_for_airdrop = state.minted_for_airdrop.add(Uint128::new(amount));
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "mint_for_airdrop")
        .add_attribute("denom", state.denom)
        .add_attribute("amount", amount.to_string())
        .add_message(msg))
}

fn receive_airdrop(deps: DepsMut, info: MessageInfo) -> Result<Response<CoreumMsg>, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    if state.minted_for_airdrop < state.airdrop_amount {
        return Err(ContractError::CustomError {
            val: "not enough minted".into(),
        });
    }
    let send_msg = cosmwasm_std::BankMsg::Send {
        to_address: info.sender.into(),
        amount: vec![Coin {
            amount: state.airdrop_amount,
            denom: state.denom.clone(),
        }],
    };

    state.minted_for_airdrop = state.minted_for_airdrop.sub(state.airdrop_amount);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "receive_airdrop")
        .add_attribute("denom", state.denom)
        .add_attribute("amount", state.airdrop_amount.to_string())
        .add_message(send_msg))
}

// ********** Queries **********

fn token(deps: Deps<CoreumQueries>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let request: QueryRequest<CoreumQueries> =
        CoreumQueries::AssetFT(assetft::Query::Token { denom: state.denom }).into();
    let res: assetft::TokenResponse = deps.querier.query(&request)?;
    to_binary(&res)
}

fn minted_for_airdrop(deps: Deps<CoreumQueries>) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let res = AmountResponse {
        amount: state.minted_for_airdrop,
    };
    to_binary(&res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, CosmosMsg};
    use coreum_wasm_sdk::assetft::TokenResponse;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            symbol: "SYM".to_string(),
            subunit: "SUB".to_string(),
            precision: 2,
            initial_amount: Uint128::new(1000),
            airdrop_amount: Uint128::new(100),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        println!("RESULT: {:?}", res);

        // it works
        assert_eq!(1, res.messages.len());

        // Initial state
        let state = STATE.load(deps.as_ref().storage).unwrap();
        assert_eq!(state.owner, "creator".to_string());
        assert_eq!(state.denom, "sub-cosmos2contract".to_string());
        assert_eq!(state.minted_for_airdrop, Uint128::new(1000));
        assert_eq!(state.airdrop_amount, Uint128::new(100));
    }

    #[test]
    fn mint_for_airdrop_works() {
        // let mut deps = mock_dependencies(&[]);
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            symbol: "SYM".to_string(),
            subunit: "SUB".to_string(),
            precision: 2,
            initial_amount: Uint128::new(1000),
            airdrop_amount: Uint128::new(100),
        };
        let info = mock_info("creator", &coins(2, "token"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Mint some coins for airdrop
        let execute_msg = ExecuteMsg::MintForAirdrop { amount: 100 };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), execute_msg).unwrap();

        println!("RESULT: {:?}", res);

        // it works
        assert_eq!(1, res.messages.len());

        // Check if the minted amount is correct
        let state = STATE.load(deps.as_ref().storage).unwrap();
        assert_eq!(state.minted_for_airdrop, Uint128::new(1100));
    }

    #[test]
    fn receive_airdrop_works() {
        // let mut deps = mock_dependencies(&[]);
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            symbol: "SYM".to_string(),
            subunit: "SUB".to_string(),
            precision: 2,
            initial_amount: Uint128::new(1000),
            airdrop_amount: Uint128::new(100),
        };
        let info = mock_info("creator", &coins(2, "token"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Receive airdrop
        let execute_msg = ExecuteMsg::ReceiveAirdrop {};
        let res = execute(deps.as_mut(), mock_env(), info.clone(), execute_msg).unwrap();

        println!("RESULT: {:?}", res);

        // it works
        assert_eq!(1, res.messages.len());

        // Check if the airdrop amount is correct
        let state = STATE.load(deps.as_ref().storage).unwrap();
        assert_eq!(state.minted_for_airdrop, Uint128::new(900));
    }

    // #[test]
    // fn query_token_works() -> Result<(), Box<dyn std::error::Error>> {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg {
    //         symbol: "SYM".to_string(),
    //         subunit: "SUB".to_string(),
    //         precision: 2,
    //         initial_amount: Uint128::new(1000),
    //         airdrop_amount: Uint128::new(100),
    //     };

    //     let info = mock_info("creator", &coins(2, "token"));
    //     instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    //     // load the state
    //     let state = STATE.load(deps.as_ref().storage).unwrap();

    //     // Query token
    //     let query_msg = QueryMsg::Token {};
    //     // let querier = deps.as_ref().querier.query_wasm_smart(state.denom.clone(), &assetft::Query::Token { denom: state.denom.clone() })?;
    //     let querier = deps.as_ref().querier.query_wasm_smart(state.denom.clone(), &assetft::Query::Token { denom: state.denom.clone() })?;

    //     let res: TokenResponse = from_binary(&query(querier, mock_env(), query_msg).unwrap()).unwrap();

    //     println!("RESULT: {:?}", res);

    //     // it works
    //     assert_eq!(res.token.symbol, "SYM");
    //     assert_eq!(res.token.subunit, "SUB");

    //     Ok(())
    // }




    // // #[test]
    // fn query_minted_for_airdrop_works() {
    //     // let mut deps = mock_dependencies(&[]);
    //     let mut deps = mock_dependencies();
    //     let msg = InstantiateMsg {
    //         symbol: "SYM".to_string(),
    //         subunit: "SUB".to_string(),
    //         precision: 2,
    //         initial_amount: Uint128::new(1000),
    //         airdrop_amount: Uint128::new(100),
    //     };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    //     // Query minted for airdrop
    //     let query_msg = QueryMsg::MintedForAirdrop {};
    //     let res: AmountResponse = from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();

    //     // it works
    //     assert_eq!(res.amount, Uint128::new(1000));
    // }
}
