use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg, Decimal, QueryRequest, WasmQuery
};
use oraidex::asset::{AssetInfo};
use oraidex::router::{OraiswapExecuteMsg, SwapOperation};
use crate::error::ContractError;
use crate::msg::{ExecuteMsgOperations, InstantiateMsgOperations, QueryMsgOperations};
use crate::state::{Config, save_config, load_config};
use oraidex::querier::{query_token_balance};
use lottery::msg::{ExecuteMsg, QueryMsg as QueryMsg_ticket};
use cw20::Cw20ExecuteMsg;



pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsgOperations,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: msg.owner,
        lottery_contract: msg.lottery_contract,
        oraiswap_router: msg.oraiswap_router,
    };

    save_config(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", config.owner.to_string())
        .add_attribute("lottery_contract", config.lottery_contract.to_string())
        .add_attribute("oraiswap_router", config.oraiswap_router.to_string()))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsgOperations,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsgOperations::BuyTicket { amount } => try_buy_ticket(deps, env, info, amount),
    }
}

fn try_buy_ticket(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    // Prepare swap operation
    let swap_operations = vec![SwapOperation::OraiSwap {
        offer_asset_info: AssetInfo::NativeToken { denom: "orai".to_string() },
        ask_asset_info: AssetInfo::Token { contract_addr: config.lottery_contract.clone() },
    }];

    let swap_msgs = query_swap_operations_msg(
        deps.as_ref(),
        env.clone(),
        env.contract.address.clone(),
        info.sender.clone(),
        Some(amount),
        swap_operations,
        Some(Uint128::new(719577)), 
        Some(env.contract.address.clone()),
        None,
    )?;

    // Ensure sufficient funds and return change if needed
    let mut messages = swap_msgs;
    let ticket_price = query_ticket_price_from_lottery_contract(deps.as_ref(), config.lottery_contract.clone())?;
    if amount < ticket_price {
        return Err(ContractError::InvalidFunds {});
    } else if amount > ticket_price {
        let change = amount - ticket_price;
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin { denom: "orai".to_string(), amount: change }],
        }));
    }

    // Prepare buy ticket message
    let buy_ticket_msg = WasmMsg::Execute {
        contract_addr: config.lottery_contract.to_string(),
        msg: to_binary(&ExecuteMsg::BuyTicket {})?,
        funds: vec![Coin { denom: "orai".to_string(), amount: ticket_price }],
    };

    messages.push(CosmosMsg::Wasm(buy_ticket_msg));

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "swap_and_buy_ticket")
        .add_attribute("amount", amount.to_string()))
}

fn query_swap_operations_msg(
    deps: Deps,
    _env: Env,
    executor_addr: Addr,
    sender: Addr,
    amount: Option<Uint128>,
    operations: Vec<SwapOperation>,
    minimum_receive: Option<Uint128>,
    to: Option<Addr>,
    half: Option<bool>,
) -> StdResult<Vec<CosmosMsg>> {
    if operations.is_empty() {
        return Err(StdError::GenericErr {
            msg: "Swap operations is empty!!!".to_string(),
        });
    }

    let config = load_config(deps.storage)?;
    let swap_router = config.oraiswap_router.clone();
    let offer_asset = operations[0].get_offer_asset_info();

    let mut messages: Vec<CosmosMsg> = vec![];

    match offer_asset {
        AssetInfo::NativeToken { denom } => {
            let amount = amount.unwrap_or_else(|| {
                let coin = deps
                    .querier
                    .query_balance(executor_addr.to_string(), denom.clone())
                    .unwrap();
                if half == Some(true) && sender == executor_addr {
                    Decimal::from_ratio(coin.amount, 2u128) * Uint128::one()
                } else {
                    coin.amount
                }
            });

            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: swap_router.to_string(),
                msg: to_binary(&OraiswapExecuteMsg::ExecuteSwapOperations {
                    operations,
                    minimum_receive,
                    to,
                })?,
                funds: vec![Coin { denom, amount }],
            }));
        }

        AssetInfo::Token { contract_addr } => {
            let amount = amount.unwrap_or_else(|| {
                let swap_amount =
                    query_token_balance(&deps.querier, contract_addr.clone(), sender.clone())
                        .unwrap();
                if half == Some(true) {
                    Decimal::from_ratio(swap_amount, 2u128) * Uint128::one()
                } else {
                    swap_amount
                }
            });

            if sender != executor_addr {
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                        owner: sender.to_string(),
                        recipient: executor_addr.to_string(),
                        amount,
                    })?,
                    funds: vec![],
                }))
            }

            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: swap_router.to_string(),
                    amount,
                    msg: to_binary(&OraiswapExecuteMsg::ExecuteSwapOperations {
                        operations,
                        minimum_receive,
                        to,
                    })?,
                })?,
                funds: vec![],
            }));
        }
    };
    Ok(messages)
}


fn query_ticket_price_from_lottery_contract(
    deps: Deps,
    lottery_contract: Addr,
) -> Result<Uint128, ContractError> {
    let query_msg = QueryMsg_ticket::GetTicketPrice {};
    let res: Coin = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: lottery_contract.to_string(),
        msg: to_binary(&query_msg)?,
    }))?;

    Ok(res.amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr, Coin, Uint128, CosmosMsg, WasmMsg, to_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsgOperations {
            owner: Addr::unchecked("owner"),
            lottery_contract: Addr::unchecked("lottery_contract"),
            oraiswap_router: Addr::unchecked("oraiswap_router"),
        };
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config = load_config(&deps.storage).unwrap();
        assert_eq!(config.owner, Addr::unchecked("owner"));
        assert_eq!(config.lottery_contract, Addr::unchecked("lottery_contract"));
        assert_eq!(config.oraiswap_router, Addr::unchecked("oraiswap_router"));
    }
}