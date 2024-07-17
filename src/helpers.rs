use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, StdResult, WasmMsg, Uint128,
};


use lottery::msg::ExecuteMsg;

pub fn buy_ticket_msg(lottery_contract: Addr) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lottery_contract.to_string(),
        msg: to_binary(&ExecuteMsg::BuyTicket {})?,
        funds: vec![],
    }))
}

pub fn transfer_token_msg(token_contract: Addr, recipient: Addr, amount: Uint128) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract.to_string(),
        msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        })?,
        funds: vec![],
    }))
}