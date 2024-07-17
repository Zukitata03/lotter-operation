use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use oraidex::router::SwapOperation;
#[cw_serde]
pub struct InstantiateMsgOperations {
    pub owner: Addr,
    pub lottery_contract: Addr,
    pub oraiswap_router: Addr,
}

#[cw_serde]
pub enum ExecuteMsgOperations {
    BuyTicket { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgOperations {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub lottery_contract: String,
    pub oraiswap_router: String,
}



#[cw_serde]
pub enum Operations {
    SwapOperations {
        executor_addr: Addr,
        sender: Addr,
        amount: Option<Uint128>,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<Addr>,
        half: Option<bool>,
    },


    BuyTicketOperations {
        amount: Uint128,
    }
}