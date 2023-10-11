use aurans_name::state::Metadata;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, SubMsg, Timestamp, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::price::{calc_price, check_fee};
use crate::state::{Config, Verifier, CONFIG, NAME_CONTRACT, PRICE_INFO, VERIFIER};
use crate::verify::verify_signature;

use serde_json::json;

use aurans_name::msg::InstantiateMsg as NameInstantiateMsg;

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

    for (l, price) in &msg.prices {
        PRICE_INFO.save(deps.storage, *l, &price)?;
    }

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
        .add_attribute("backend_pubkey", msg.backend_pubkey.to_string())
        .add_attribute("name_code_id", msg.name_code_id.to_string())
        .add_attribute("resolver_code_id", msg.resolver_code_id.to_string())
        .add_attribute(
            "prices",
            msg.prices
                .iter()
                .map(|(l, price)| format!("{}:{}", l, price.to_string()))
                .collect::<Vec<String>>()
                .join("-"),
        ))
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
        ExecuteMsg::UpdatePrices { prices } => execute_update_prices(deps, env, info, prices),
        ExecuteMsg::UpdateVerifier { backend_pubkey } => {
            execute_update_verifier(deps, env, info, backend_pubkey)
        }
        ExecuteMsg::Register {
            name,
            bech32_prefixes,
            backend_signature,
            metadata,
        } => execute_register(
            deps,
            env,
            info,
            name,
            bech32_prefixes,
            backend_signature,
            metadata,
        ),
        ExecuteMsg::ExtendExpires {
            name,
            old_expires,
            new_expires,
        } => execute_extend_expires(deps, env, info, name, old_expires, new_expires),
    }
}

fn execute_extend_expires(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    old_expires: Timestamp,
    new_expires: Timestamp,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    let msg = aurans_name::ExecuteMsg::Extension {
        msg: aurans_name::NameExecuteMsg::ExtendExpires {
            token_id: format!("{}@{}", name.clone(), old_expires.seconds()),
            new_expires: new_expires,
        },
    };
    let extend_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };
    Ok(Response::new()
        .add_message(extend_msg)
        .add_attribute("action", "extend_expires")
        .add_attribute("name", name)
        .add_attribute("new_expires", new_expires.to_string()))
}

fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    bech32_prefixes: Vec<String>,
    backend_signature: Binary,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // If not owner, check verification msg
    if config.admin != info.sender {
        let verify_msg_json = json!({
            "name": name,
            "bech32_prefixes": bech32_prefixes,
            "sender": info.sender.to_string(),
            "chain_id": env.block.chain_id,
        });
        let verify_msg_str =
            serde_json::to_string(&verify_msg_json).map_err(|_| ContractError::SerdeError)?;

        let verifier = VERIFIER.load(deps.storage)?;

        verify_signature(
            deps.as_ref(),
            &verify_msg_str,
            &backend_signature.as_slice(),
            &verifier.backend_pubkey,
        )?;
    }

    // Check fee
    let fee = calc_price(deps.as_ref(), &name)?;
    check_fee(fee, &info.funds)?;

    // Call mint msg
    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    let mint_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Mint {
            token_id: name.clone(),
            owner: info.sender.to_string(),
            token_uri: None,
            extension: Metadata {
                image: metadata.image,
                image_data: metadata.image_data,
                external_url: metadata.external_url,
                description: metadata.description,
                name: metadata.name,
                attributes: metadata.attributes,
                background_color: metadata.background_color,
                animation_url: metadata.animation_url,
                youtube_url: metadata.youtube_url,
                royalty_percentage: metadata.royalty_percentage,
                royalty_payment_address: metadata.royalty_payment_address,
                bech32_prefixes: bech32_prefixes.clone(),
            },
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "register")
        .add_attribute("name", name)
        .add_attribute("bech32_prefixes", bech32_prefixes.join("-"))
        .add_attribute("backend_signature", backend_signature.to_string()))
}

fn execute_update_verifier(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    backend_pubkey: Binary,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    VERIFIER.save(
        deps.storage,
        &Verifier {
            backend_pubkey: backend_pubkey.clone(),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "update_verifier")
        .add_attribute("backend_pubkey", backend_pubkey.to_string()))
}

fn execute_update_prices(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    prices: Vec<(u8, Coin)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    for (l, price) in &prices {
        PRICE_INFO.update(deps.storage, *l, |d| -> StdResult<Coin> {
            match d {
                Some(coin) => Ok(coin),
                None => Ok(Coin {
                    denom: price.denom.clone(),
                    amount: price.amount,
                }),
            }
        })?;
    }

    Ok(Response::new()
        .add_attribute("action", "update_prices")
        .add_attribute(
            "prices",
            prices
                .iter()
                .map(|(l, price)| format!("{}:{}", l, price.to_string()))
                .collect::<Vec<String>>()
                .join("-"),
        ))
}

fn execute_update_config(
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
