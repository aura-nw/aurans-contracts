use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub name_contract: Addr,
}

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub name_contract: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: String,
        name_contract: String,
    },
    SetRecord {
        name: String,
        bech32_prefix: String,
        address: String,
    },
    DeleteRecord {
        name: String,
        bech32_prefix: String,
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

    #[returns(AddressesOfResponse)]
    AddressOf {
        primary_name: String,
        bech32_prefix: String,
    },

    #[returns(AddressesOfResponse)]
    AddressesOf { primary_name: String },

    #[returns(NamesResponse)]
    Names { owner: String, limit: Option<u32> },
}

#[cw_serde]
pub struct AddressOfResponse {
    pub address: String,
}

#[cw_serde]
pub struct AddressesOfResponse {
    // A list of pair (address, bech32_prefix)
    pub addresses: Vec<(String, String)>,
}

#[cw_serde]
pub struct NamesResponse {
    pub names: Vec<String>,
}
