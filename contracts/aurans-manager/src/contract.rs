use aurans_name::msg::InstantiateMsg as NameInstantiateMsg;
use aurans_resolver::state::NAME_CONTRACT;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::state::{Config, PriceInfo, Verifier, CONFIG, PRICE_INFO, VERIFIER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
        name_code_id: msg.name_code_id.clone(),
        resolver_code_id: msg.resolver_code_id.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    let price_info = PriceInfo {
        base_price: msg.base_price.clone(),
    };
    PRICE_INFO.save(deps.storage, &price_info)?;

    let verifier = Verifier {
        backend_pubkey: msg.backend_pubkey.clone(),
    };
    VERIFIER.save(deps.storage, &verifier)?;

    let name_ins_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: config.name_code_id,
        msg: to_binary(&NameInstantiateMsg {
            admin: config.admin.to_string(),
            minter: env.contract.address.to_string(),
            resolver_code_id: config.resolver_code_id,
        })?,
        funds: vec![],
        label: "name".to_owned(),
    });

    let name_sub_msg = SubMsg {
        id: 1,
        msg: name_ins_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_submessage(name_sub_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("base_price", msg.base_price.to_string())
        .add_attribute("backend_pubkey", msg.backend_pubkey.to_string())
        .add_attribute("name_code_id", msg.name_code_id.to_string())
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
    let _api = deps.api;
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            name_code_id,
            resolver_code_id,
        } => execute_update_config(deps, env, info, admin, name_code_id, resolver_code_id),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
    name_code_id: u64,
    resolver_code_id: u64,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
        name_code_id,
        resolver_code_id,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", admin.to_string())
        .add_attribute("name_code_id", name_code_id.to_string())
        .add_attribute("resolver_code_id", resolver_code_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let name_contract = deps.api.addr_validate(&reply.contract_address)?;
    NAME_CONTRACT.save(deps.storage, &name_contract)?;

    Ok(Response::new().add_attribute("name_contract", name_contract))
}
