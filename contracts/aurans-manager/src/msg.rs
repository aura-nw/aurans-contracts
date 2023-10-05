use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin};

use crate::state::Config;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub prices: Vec<(String, Coin)>,
    pub backend_pubkey: Binary,
    pub name_code_id: u64,
    pub resolver_code_id: u64,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: String,
        name_code_id: u64,
        resolver_code_id: u64,
    },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}
