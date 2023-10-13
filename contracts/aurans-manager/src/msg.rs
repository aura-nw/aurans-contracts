use aurans_name::state::Metadata;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Timestamp};

use crate::state::Config;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub prices: Vec<(u8, Coin)>,
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
    UpdatePrices {
        prices: Vec<(u8, Coin)>,
    },
    UpdateVerifier {
        backend_pubkey: Binary,
    },
    Register {
        name: String,
        backend_signature: Binary,
        metadata: Metadata,
    },
    ExtendExpires {
        name: String,
        backend_signature: Binary,
        old_expires: Timestamp,
        new_expires: Timestamp,
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

#[cw_serde]
pub enum VerifyMsg {
    Register {
        name: String,
        sender: String,
        chain_id: String,
        bech32_prefixes: Vec<String>,
        expires: u64,
    },
    ExtendExpires {
        name: String,
        sender: String,
        chain_id: String,
        old_expires: u64,
        new_expires: u64,
    },
}
