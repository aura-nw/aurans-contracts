#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw721_base::Cw721Contract;
use cw721_base::ExecuteMsg::{
    Approve, ApproveAll, Burn, Extension as EExtension, Mint, Revoke, RevokeAll, SendNft,
    TransferNft, UpdateOwnership,
};

use cw721_base::QueryMsg::Extension as QExtension;

use aurans_resolver::ExecuteMsg::{DeleteNames, UpdateRecord};
use cw721_base::state::TokenInfo;
use cw_utils::Expiration;

use crate::error::ContractError;
use crate::state::{Config, Metadata, CONFIG};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, NameExecuteMsg, NameQueryMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-name";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AURANS_NAME: &str = "aurans-name";
const AURANS_SYMBOL: &str = "aurans";

/// This contract extends the Cw721 contract from CosmWasm to create non-fungible tokens (NFTs)
/// that represent unique names. Each name is represented as a unique NFT.
/// It inherits and builds upon the functionality provided by the Cw721 contract.
pub type NameCw721<'a> = Cw721Contract<'a, Metadata, Empty, NameExecuteMsg, NameQueryMsg>;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // save contract config
    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?,
        resolver_contract: deps.api.addr_validate(&msg.resolver_contract)?,
    };
    CONFIG.save(deps.storage, &config)?;

    let name_cw721 = NameCw721::default();

    // Save contract info
    let info = ContractInfoResponse {
        name: AURANS_NAME.to_owned(),
        symbol: AURANS_SYMBOL.to_owned(),
    };
    name_cw721.contract_info.save(deps.storage, &info)?;

    // initialize owner
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.minter))?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", &msg.admin.to_string())
        .add_attribute("resolver_contract", &msg.resolver_contract.to_string())
        .add_attribute("minter", msg.minter.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => execute_mint(deps, env, info, token_id, owner, token_uri, extension),
        EExtension { msg } => match msg {
            NameExecuteMsg::UpdateConfig {
                admin,
                resolver_contract,
            } => execute_update_config(deps, env, info, admin, resolver_contract),
            NameExecuteMsg::ExtendTTL {
                token_id,
                new_expires,
            } => execute_extend_ttl(deps, env, info, token_id, new_expires),
            NameExecuteMsg::EvictBatch { token_ids } => {
                execute_evict_batch(deps, env, info, token_ids)
            }
            NameExecuteMsg::UpdateResolver { resolver } => {
                execute_update_resolver(deps, env, info, resolver)
            }
        },
        msg @ Approve { .. }
        | msg @ ApproveAll { .. }
        | msg @ Burn { .. }
        | msg @ UpdateOwnership { .. }
        | msg @ Revoke { .. }
        | msg @ RevokeAll { .. } => {
            let name_cw721 = NameCw721::default();
            name_cw721.execute(deps, env, info, msg).map_err(Into::into)
        }
    }
}

fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    owner: String,
    token_uri: Option<String>,
    extension: Metadata,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)
        .map_err(|_| ContractError::Unauthorized {})?;

    let token = TokenInfo {
        owner: deps.api.addr_validate(&owner)?,
        approvals: vec![],
        token_uri: token_uri,
        extension: extension.clone(),
    };
    let name_cw721 = NameCw721::default();
    name_cw721
        .tokens
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(ContractError::CW721Base(
                cw721_base::ContractError::Claimed {},
            )),
            None => Ok(token.clone()),
        })?;
    name_cw721.increment_tokens(deps.storage)?;

    let metadata = token.extension;
    let update_record = UpdateRecord {
        name: token_id.clone(),
        list_bech32_prefix: metadata.bech32_prefix_registed,
        address: owner.clone(),
    };
    let config = CONFIG.load(deps.storage)?;
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: config.resolver_contract.to_string(),
        msg: to_binary(&update_record)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(update_resolver_msg)
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", owner)
        .add_attribute("token_id", token_id)
        .add_attribute(
            "bech32_prefix_registed",
            extension
                .bech32_prefix_registed
                .into_iter()
                .collect::<String>(),
        )
        .add_attribute("ttl", extension.expires.to_string()))
}

fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let name_cw721 = NameCw721::default();
    let token = name_cw721._transfer_nft(deps, &env, &info, &recipient, &token_id)?;
    let metadata = token.extension;
    let update_record = UpdateRecord {
        name: token_id.clone(),
        list_bech32_prefix: metadata.bech32_prefix_registed,
        address: recipient.clone(),
    };
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: config.resolver_contract.to_string(),
        msg: to_binary(&update_record)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(update_resolver_msg)
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id))
}

fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let name_cw721 = NameCw721::default();
    let token = name_cw721._transfer_nft(deps, &env, &info, &contract, &token_id)?;
    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };
    let metadata = token.extension;
    let update_record = UpdateRecord {
        name: token_id.clone(),
        list_bech32_prefix: metadata.bech32_prefix_registed,
        address: contract.clone(),
    };
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: config.resolver_contract.to_string(),
        msg: to_binary(&update_record)?,
        funds: vec![],
    };
    // Send message and update resolver
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_message(update_resolver_msg)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

fn execute_update_resolver(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    resolver: String,
) -> Result<Response, ContractError> {
    // only contract admin can update resolver
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config with new resolver
    let new_config = Config {
        admin: config.admin,
        resolver_contract: deps.api.addr_validate(&resolver)?,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_resolver")
        .add_attribute("resolver", resolver))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
    resolver_contract: String,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
        resolver_contract: deps.api.addr_validate(&resolver_contract)?,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", &admin)
        .add_attribute("resolver_contract", &resolver_contract))
}

fn execute_extend_ttl(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    expires: Expiration,
) -> Result<Response, ContractError> {
    let name_cw721 = NameCw721::default();
    let mut token = name_cw721.tokens.load(deps.storage, &token_id)?;
    name_cw721.check_can_send(deps.as_ref(), &env, &info, &token)?;
    let old_metadata = token.extension.clone();
    token.extension.bech32_prefix_registed = old_metadata.bech32_prefix_registed;
    token.extension.expires = expires;

    name_cw721.tokens.save(deps.storage, &token_id, &token)?;
    Ok(Response::new()
        .add_attribute("action", "extend_ttl")
        .add_attribute("token_id", &token_id)
        .add_attribute("new_ttl", expires.to_string()))
}

const DEFAULT_LIMIT_BACTH: usize = 10;

fn execute_evict_batch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    if token_ids.len() > DEFAULT_LIMIT_BACTH {
        return Err(ContractError::BatchTooLong {});
    }
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let name_cw721 = NameCw721::default();
    for token_id in &token_ids {
        let token = name_cw721.tokens.load(deps.storage, &token_id)?;
        name_cw721.check_can_send(deps.as_ref(), &env, &info, &token)?;
        name_cw721.tokens.remove(deps.storage, &token_id)?;
        name_cw721.decrement_tokens(deps.storage)?;
    }
    // Delete records has burn to resolver
    let delete_names = DeleteNames {
        names: token_ids.clone(),
    };
    let config = CONFIG.load(deps.storage)?;
    let delete_resolver_msg = WasmMsg::Execute {
        contract_addr: config.resolver_contract.to_string(),
        msg: to_binary(&delete_names)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(delete_resolver_msg)
        .add_attribute("action", "evict_batch")
        .add_attribute("sender", &info.sender)
        .add_attribute("token_ids", token_ids.into_iter().collect::<String>()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Default query by cw721
        QExtension { msg } => match msg {
            NameQueryMsg::Config {} => to_binary(&query_config(deps)?),
        },
        _ => {
            let name_cw721 = NameCw721::default();
            name_cw721.query(deps, env, msg)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}
