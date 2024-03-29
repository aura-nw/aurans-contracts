use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::Config;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: String,
    },
    UpdateNameContract {
        name_contract: String,
    },
    UpdateRecord {
        name: String,
        bech32_prefixes: Vec<String>,
        address: String,
    },
    DeleteNames {
        names: Vec<String>,
    },
    AddIgnoreAddress {
        address: String,
    },
    RemoveIgnoreAddress {
        address: String,
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

    #[returns(Addr)]
    NameContract {},

    #[returns(bool)]
    IsIgnoreAddress { address: String },

    #[returns(AddressResponse)]
    AddressOf {
        primary_name: String,
        bech32_prefix: String,
    },

    #[returns(Vec<AddressResponse>)]
    AllAddressesOf {
        primary_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(NamesResponse)]
    Names {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct AddressResponse {
    pub address: String,
    pub bech32_prefix: String,
}

#[cw_serde]
pub struct NamesResponse {
    pub names: Vec<String>,
}
