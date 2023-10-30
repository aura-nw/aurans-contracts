#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw721_base::Cw721Contract;
use cw721_base::ExecuteMsg::{
    Approve, ApproveAll, Burn, Extension as EExtension, Mint, Revoke, RevokeAll, SendNft,
    TransferNft, UpdateOwnership,
};

use cw721_base::QueryMsg::Extension as QExtension;

use aurans_resolver::msg::InstantiateMsg as ResolverInstantiateMsg;
use aurans_resolver::ExecuteMsg::{DeleteNames, UpdateRecord};
use cw721_base::state::TokenInfo;
use cw_utils::parse_reply_instantiate_data;
use std::vec;

use crate::error::ContractError;
use crate::state::{Config, Metadata, Resolver, CONFIG, RESOLVER};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, NameExecuteMsg, NameQueryMsg, QueryMsg};
use crate::util::extract_name_from_token_id;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-name";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const AURANS_NAME: &str = "aurans-name";
const AURANS_SYMBOL: &str = "ans";

const DEFAULT_LIMIT_BACTH: usize = 10;

/// This contract extends the Cw721 contract from CosmWasm to create non-fungible tokens (NFTs)
/// that represent unique names. Each name is represented as a unique NFT.
/// It inherits and builds upon the functionality provided by the Cw721 contract.
pub type NameCw721<'a> = Cw721Contract<'a, Metadata, Empty, NameExecuteMsg, NameQueryMsg>;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // save contract config
    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?,
        minter: deps.api.addr_validate(&msg.minter)?,
    };
    CONFIG.save(deps.storage, &config)?;

    let name_cw721 = NameCw721::default();

    // Save contract info
    let info = ContractInfoResponse {
        name: AURANS_NAME.to_owned(),
        symbol: AURANS_SYMBOL.to_owned(),
    };
    name_cw721.contract_info.save(deps.storage, &info)?;

    let resolver_ins_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.resolver_code_id,
        msg: to_binary(&ResolverInstantiateMsg {
            admin: config.admin.to_string(),
        })?,
        funds: vec![],
        label: "aurans-resolver".to_owned(),
    });

    let resolver_sub_msg = SubMsg {
        id: 1,
        msg: resolver_ins_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(resolver_sub_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("admin", &msg.admin.to_string())
        .add_attribute("minter", msg.minter.to_string())
        .add_attribute("resolver_code_id", msg.resolver_code_id.to_string()))
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
        Burn { token_id } => execute_burn(deps, env, info, token_id),
        EExtension { msg } => match msg {
            NameExecuteMsg::UpdateConfig { admin, minter } => {
                execute_update_config(deps, env, info, admin, minter)
            }
            NameExecuteMsg::BurnBatch { token_ids } => {
                execute_burn_batch(deps, env, info, token_ids)
            }
            NameExecuteMsg::UpdateResolver { resolver } => {
                execute_update_resolver(deps, env, info, resolver)
            }
        },
        msg @ Approve { .. }
        | msg @ ApproveAll { .. }
        | msg @ UpdateOwnership { .. }
        | msg @ Revoke { .. }
        | msg @ RevokeAll { .. } => {
            let name_cw721 = NameCw721::default();
            name_cw721.execute(deps, env, info, msg).map_err(Into::into)
        }
    }
}

// In cw721 implementation it check can send of token.
// So we need custom burn execution, allow minter and owner can burn the token
fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    // Require sender is minter or admin
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin && info.sender != config.minter {
        return Err(ContractError::Unauthorized {
            sender: info.sender.clone().to_string(),
        });
    }

    let name_cw721 = NameCw721::default();
    name_cw721.tokens.remove(deps.storage, &token_id)?;
    name_cw721.decrement_tokens(deps.storage)?;

    let (name, _) = extract_name_from_token_id(&token_id)?;

    // Delete name from resolver
    let resolver = RESOLVER.load(deps.storage)?;
    let delete_names = DeleteNames {
        names: vec![name.to_string()],
    };
    let delete_names_msg = WasmMsg::Execute {
        contract_addr: resolver.address.to_string(),
        msg: to_binary(&delete_names)?,
        funds: vec![],
    };
    Ok(Response::new()
        .add_message(delete_names_msg)
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id.to_string()))
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
    let resolver = RESOLVER.load(deps.as_ref().storage)?;
    let name_cw721 = NameCw721::default();

    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&owner)?,
        approvals: vec![],
        token_uri: token_uri.clone(),
        extension: extension.clone(),
    };
    name_cw721
        .tokens
        .update(deps.storage, &token_id, |old| match old {
            Some(_) => Err(ContractError::CW721Base(
                cw721_base::ContractError::Claimed {},
            )),
            None => Ok(token),
        })?;

    name_cw721.increment_tokens(deps.storage)?;

    let (name, expires) = extract_name_from_token_id(token_id.as_ref())?;
    let update_record = UpdateRecord {
        name: name.to_owned(),
        bech32_prefixes: extension.clone().bech32_prefixes,
        address: owner.clone(),
    };
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: resolver.address.to_string(),
        msg: to_binary(&update_record)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(update_resolver_msg)
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", owner)
        .add_attribute("token_id", token_id)
        .add_attribute("expires", expires.to_string())
        .add_attribute("bech32_prefixes", extension.bech32_prefixes.join(",")))
}

fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let resolver = RESOLVER.load(deps.as_ref().storage)?;
    let name_cw721 = NameCw721::default();
    let token = name_cw721._transfer_nft(deps, &env, &info, &recipient, &token_id)?;
    let metadata = token.extension;
    let (name, _) = extract_name_from_token_id(token_id.as_ref())?;
    let update_record = UpdateRecord {
        name: name.to_owned(),
        bech32_prefixes: metadata.bech32_prefixes,
        address: recipient.clone(),
    };
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: resolver.address.to_string(),
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
    let resolver = RESOLVER.load(deps.as_ref().storage)?;
    let name_cw721 = NameCw721::default();
    let token = name_cw721._transfer_nft(deps, &env, &info, &contract, &token_id)?;
    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };
    let metadata = token.extension;
    let (name, _) = extract_name_from_token_id(token_id.as_ref())?;
    let update_record = UpdateRecord {
        name: name.to_owned(),
        bech32_prefixes: metadata.bech32_prefixes,
        address: contract.clone(),
    };
    let update_resolver_msg = WasmMsg::Execute {
        contract_addr: resolver.address.to_string(),
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
        return Err(ContractError::Unauthorized {
            sender: info.sender.clone().to_string(),
        });
    }

    let r = Resolver {
        address: deps.api.addr_validate(&resolver)?,
    };
    RESOLVER.save(deps.storage, &r)?;

    Ok(Response::new()
        .add_attribute("action", "update_resolver")
        .add_attribute("resolver", resolver))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
    minter: String,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {
            sender: info.sender.clone().to_string(),
        });
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
        minter: deps.api.addr_validate(&minter)?,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", admin)
        .add_attribute("minter", minter))
}

// REQUIRED: sender must be admin
fn execute_burn_batch(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    if token_ids.len() > DEFAULT_LIMIT_BACTH {
        return Err(ContractError::BatchTooLong {});
    }
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {
            sender: info.sender.clone().to_string(),
        });
    }

    let name_cw721 = NameCw721::default();
    for token_id in &token_ids {
        name_cw721.tokens.remove(deps.storage, &token_id)?;
        name_cw721.decrement_tokens(deps.storage)?;
    }
    // Delete records has burn to resolver
    let mut names: Vec<String> = Vec::new();
    for token_id in &token_ids {
        let (name, _) = extract_name_from_token_id(token_id)?;
        names.push(name.to_owned());
    }
    let delete_names = DeleteNames { names };
    let resolver = RESOLVER.load(deps.as_ref().storage)?;
    let delete_resolver_msg = WasmMsg::Execute {
        contract_addr: resolver.address.to_string(),
        msg: to_binary(&delete_names)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(delete_resolver_msg)
        .add_attribute("action", "burn_batch")
        .add_attribute("sender", &info.sender)
        .add_attribute("token_ids", token_ids.into_iter().collect::<String>()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Default query by cw721
        QExtension { msg } => match msg {
            NameQueryMsg::Config {} => to_binary(&query_config(deps)?),
            NameQueryMsg::Resolver {} => to_binary(&query_resolver(deps)?),
        },
        _ => {
            let name_cw721 = NameCw721::default();
            name_cw721.query(deps, env, msg)
        }
    }
}

fn query_resolver(deps: Deps) -> StdResult<Resolver> {
    RESOLVER.load(deps.storage)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let resolver_address = reply.contract_address;
    let r = Resolver {
        address: deps.api.addr_validate(&resolver_address)?,
    };
    RESOLVER.save(deps.storage, &r)?;

    Ok(Response::new().add_attribute("resolver_address", resolver_address.to_string()))
}
