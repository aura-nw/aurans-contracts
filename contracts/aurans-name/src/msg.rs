use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::CustomMsg;

use crate::state::{Config, Metadata};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub resolver_code_id: u64,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,
}

/// Message type for `execute` entry_point
pub type ExecuteMsg = cw721_base::ExecuteMsg<Metadata, NameExecuteMsg>;

#[cw_serde]
pub enum NameExecuteMsg {
    UpdateConfig {
        admin: String,
    },
    UpdateResolver {
        resolver: String,
    },
    ExtendExpires {
        token_id: String,
        new_expires: u64,
    },
    EvictBatch {
        // Should be limit batch size
        token_ids: Vec<String>,
    },
}

/// Message type for `query` entry_point
pub type QueryMsg = cw721_base::QueryMsg<NameQueryMsg>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum NameQueryMsg {
    #[returns(Config)]
    Config {},
}

impl CustomMsg for NameExecuteMsg {}
impl CustomMsg for NameQueryMsg {}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}
