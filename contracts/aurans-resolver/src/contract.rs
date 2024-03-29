#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::StdError;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_storage_plus::KeyDeserialize;

use crate::error::ContractError;
use crate::state::IGNORE_ADDRS;
use crate::state::{records, Config, CONFIG, NAME_CONTRACT};

use crate::msg::{
    AddressResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, NamesResponse, QueryMsg,
};
use crate::util;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-resolver";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // save contract config
    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?,
    };
    CONFIG.save(deps.storage, &config)?;

    NAME_CONTRACT.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", msg.admin))
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
        ExecuteMsg::UpdateConfig { admin } => execute_update_config(deps, env, info, admin),
        ExecuteMsg::UpdateRecord {
            name,
            bech32_prefixes,
            address,
        } => execute_update_record(deps, env, info, name, bech32_prefixes, address),
        ExecuteMsg::UpdateNameContract { name_contract } => {
            execute_update_name_contract(deps, env, info, name_contract)
        }
        ExecuteMsg::DeleteNames { names } => execute_delete_names(deps, env, info, names),
        ExecuteMsg::AddIgnoreAddress { address } => {
            execute_add_ignore_address(deps, env, info, address)
        }
        ExecuteMsg::RemoveIgnoreAddress { address } => {
            execute_remove_ignore_address(deps, env, info, address)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::NameContract {} => to_binary(&query_name_contract(deps)?),
        QueryMsg::IsIgnoreAddress { address } => {
            to_binary(&query_is_ignore_address(deps, address)?)
        }
        QueryMsg::AddressOf {
            primary_name,
            bech32_prefix,
        } => to_binary(&query_address_of(deps, primary_name, bech32_prefix)?),
        QueryMsg::AllAddressesOf {
            primary_name,
            start_after,
            limit,
        } => to_binary(&query_all_addresses_of(
            deps,
            primary_name,
            start_after,
            limit,
        )?),
        QueryMsg::Names {
            owner,
            start_after,
            limit,
        } => to_binary(&query_names(deps, owner, start_after, limit)?),
    }
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", admin))
}

fn execute_add_ignore_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    IGNORE_ADDRS.save(deps.storage, &address, &true)?;
    Ok(Response::new()
        .add_attribute("action", "add_ignore_address")
        .add_attribute("address", address))
}

fn execute_remove_ignore_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if !IGNORE_ADDRS.has(deps.storage, &address) {
        return Err(ContractError::Std(StdError::NotFound {
            kind: format!("address not found in ignore addrs: {:?}", address),
        }));
    }
    IGNORE_ADDRS.remove(deps.storage, &address);
    Ok(Response::new()
        .add_attribute("action", "remove_ignore_address")
        .add_attribute("address", address))
}

fn execute_update_name_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name_contract: String,
) -> Result<Response, ContractError> {
    // only contract admin can update name contract
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let name_contract = deps.api.addr_validate(&name_contract)?;
    NAME_CONTRACT.save(deps.storage, &name_contract)?;
    Ok(Response::new()
        .add_attribute("action", "update_name_contract")
        .add_attribute("name_contract", name_contract))
}

fn execute_update_record(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    bech32_prefixes: Vec<String>,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    can_execute(deps.as_ref(), &config, &info.sender)?;

    for bech32_prefix in &bech32_prefixes {
        let bech32_addr_decoded = util::bech32_decode(&address)?;
        let bech32_addr = util::bech32_encode(bech32_prefix, &bech32_addr_decoded);
        records().save(deps.storage, (&name, &bech32_prefix), &bech32_addr)?;
    }
    Ok(Response::new()
        .add_attribute("action", "update_record")
        .add_attribute("name", &name)
        .add_attribute("bech32_prefixes", &bech32_prefixes.join(","))
        .add_attribute("address", &address))
}

fn execute_delete_names(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    names: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    can_execute(deps.as_ref(), &config, &info.sender)?;

    for name in &names {
        records().prefix(name).clear(deps.storage, None);
    }

    Ok(Response::new()
        .add_attribute("action", "delete_names")
        .add_attribute("names", names.join(",")))
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_name_contract(deps: Deps) -> StdResult<Addr> {
    NAME_CONTRACT.load(deps.storage)
}

fn query_is_ignore_address(deps: Deps, address: String) -> StdResult<bool> {
    let found = IGNORE_ADDRS.has(deps.storage, &address);
    Ok(found)
}

fn query_address_of(
    deps: Deps,
    primary_name: String,
    bech32_prefix: String,
) -> StdResult<AddressResponse> {
    let key = (primary_name.as_ref(), bech32_prefix.as_ref());
    let address = records().load(deps.storage, key)?;
    if IGNORE_ADDRS.has(deps.storage, &address) {
        return Err(StdError::generic_err("address in ignored address"));
    }
    Ok(AddressResponse {
        address,
        bech32_prefix,
    })
}

fn query_all_addresses_of(
    deps: Deps,
    primary_name: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<AddressResponse>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let mut addresses: Vec<AddressResponse> = Vec::new();

    let records = records()
        .prefix(&primary_name)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    for record in records {
        let (bech32_prefix, address) = record;
        if IGNORE_ADDRS.has(deps.storage, &address) {
            continue;
        }
        addresses.push(AddressResponse {
            address,
            bech32_prefix,
        });
    }

    Ok(addresses)
}

fn query_names(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<NamesResponse> {
    if IGNORE_ADDRS.has(deps.storage, &owner) {
        return Err(StdError::generic_err("address in ignored address"));
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let names = records()
        .idx
        .address
        .prefix(owner)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|result| {
            result
                .iter()
                .map(|key| {
                    let (name, bech32_prefix) = <(String, String)>::from_slice(key.as_bytes())?;
                    Ok(format!("{}.{}", name, bech32_prefix))
                })
                .collect::<StdResult<String>>()
        })
        .collect::<StdResult<Vec<_>>>()?;
    Ok(NamesResponse { names })
}

// Return true if sender is admin or address of name contract
fn can_execute(deps: Deps, config: &Config, sender: &Addr) -> Result<bool, ContractError> {
    if sender.to_string() == config.admin {
        return Ok(true);
    }
    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    if sender.to_string() == name_contract {
        return Ok(true);
    }

    Err(ContractError::Unauthorized {})
}
