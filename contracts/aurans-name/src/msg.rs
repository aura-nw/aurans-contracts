use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomMsg, Empty};
use cw721::{
    ApprovalResponse, ApprovalsResponse, ContractInfoResponse, NumTokensResponse,
    OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::{msg::QueryMsg as Cw721QueryMsg, MinterResponse};

use crate::state::{Config, Metadata};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub resolver_contract: String,

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
        resolver_contract: String,
    },
    ExtendTTL {
        token_id: String,
        new_ttl: u64,
    },
    EvictBatch {
        // Should be limit batch size
        token_ids: Vec<String>,
    },
}
impl CustomMsg for NameExecuteMsg {}

pub type NftInfoResponse = cw721::NftInfoResponse<Metadata>;
pub type AllNftInfoResponse = cw721::AllNftInfoResponse<Metadata>;

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },

    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },

    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },

    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(NumTokensResponse)]
    NumTokens {},

    #[returns(ContractInfoResponse)]
    ContractInfo {},

    #[returns(NftInfoResponse)]
    NftInfo { token_id: String },

    #[returns(AllNftInfoResponse)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },

    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(MinterResponse)]
    Minter {},
}

impl From<QueryMsg> for Cw721QueryMsg<Empty> {
    fn from(msg: QueryMsg) -> Cw721QueryMsg<Empty> {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::NumTokens {} => Cw721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => Cw721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => Cw721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw721QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::Minter {} => Cw721QueryMsg::Minter {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}
