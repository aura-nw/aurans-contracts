#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{records, Config, CONFIG};

use crate::msg::{
    AddressResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, NamesResponse, QueryMsg,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-resolver";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        name_contract: deps.api.addr_validate(&msg.name_contract)?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new()
            .add_attributes([("action", "instantiate"), ("admin", info.sender.as_ref())]),
    )
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
    let _api = deps.api;
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            name_contract,
        } => execute_update_config(deps, env, info, admin, name_contract),
        ExecuteMsg::UpdateRecord {
            name,
            bech32_prefix,
            address,
        } => execute_update_record(deps, env, info, name, bech32_prefix, address),
        ExecuteMsg::DeleteRecord {
            name,
            bech32_prefix,
        } => execute_delete_record(deps, env, info, name, bech32_prefix),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::AddressOf {
            primary_name,
            bech32_prefix,
        } => to_binary(&query_address_of(deps, primary_name, bech32_prefix)?),
        QueryMsg::AllAddressesOf { primary_name } => {
            to_binary(&query_all_addresses_of(deps, primary_name)?)
        }
        QueryMsg::Names { owner, limit } => to_binary(&query_names(deps, owner, limit)?),
    }
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
    name_contract: String,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
        name_contract: deps.api.addr_validate(&name_contract)?,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attributes([("action", "update_config"), ("admin", &admin)]))
}

fn execute_update_record(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    bech32_prefix: String,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !can_execute(&config, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    records().save(deps.storage, (&name, &bech32_prefix), &address)?;
    Ok(Response::new()
        .add_attribute("action", "update_record")
        .add_attribute("name", &name)
        .add_attribute("bech32_prefix", &bech32_prefix)
        .add_attribute("address", &address))
}

fn execute_delete_record(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    bech32_prefix: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !can_execute(&config, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    records().remove(deps.storage, (&name, &bech32_prefix))?;
    Ok(Response::new()
        .add_attribute("action", "delete_record")
        .add_attribute("name", &name)
        .add_attribute("bech32_prefix", &bech32_prefix))
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_address_of(
    deps: Deps,
    primary_name: String,
    bech32_prefix: String,
) -> StdResult<AddressResponse> {
    let key = (primary_name.as_ref(), bech32_prefix.as_ref());
    let addr = records().load(deps.storage, key)?;
    Ok(AddressResponse {
        address: addr,
        bech32_prefix: bech32_prefix,
    })
}

fn query_all_addresses_of(deps: Deps, primary_name: String) -> StdResult<Vec<AddressResponse>> {
    let mut addresses: Vec<AddressResponse> = Vec::new();

    let records = records()
        .prefix(&primary_name)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for record in records {
        addresses.push(AddressResponse {
            address: record.0,
            bech32_prefix: record.1,
        });
    }

    Ok(addresses)
}

fn query_names(deps: Deps, owner: String, limit: Option<u32>) -> StdResult<NamesResponse> {
    let limit = limit.unwrap_or(10) as usize;
    let names = records()
        .idx
        .address
        .prefix(owner)
        .keys(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    Ok(NamesResponse { names })
}

fn can_execute(config: &Config, sender: &Addr) -> bool {
    // Return true if sender is admin
    if config.admin == sender.to_string() {
        return true;
    }
    // Return true if sender is address of name contract
    if config.name_contract == sender.to_string() {
        return true;
    }
    false
}
