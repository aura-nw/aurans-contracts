use aurans_name::state::Metadata;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::state::{Config, Verifier};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub prices: Vec<(u8, Coin)>,
    pub backend_pubkey: Binary,
    pub name_code_id: u64,
    pub resolver_code_id: u64,
    pub max_year_register: u64,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: String,
        name_code_id: u64,
        resolver_code_id: u64,
        max_year_register: u64,
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
        extends: u64,
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
    #[returns(Verifier)]
    Verifier {},
    #[returns(PricesResponse)]
    Prices {},
    #[returns(Addr)]
    NameContract {},
}

#[cw_serde]
pub struct PricesResponse {
    pub prices: Vec<(u8, Coin)>,
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
        extends: u64,
    },
}
