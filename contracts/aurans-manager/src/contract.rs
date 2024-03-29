#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    QueryRequest, Reply, ReplyOn, Response, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::NftInfoResponse;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::price::{calc_price, check_fee};
use crate::state::{Config, Verifier, CONFIG, NAME_CONTRACT, PRICE_INFO, REGISTERS, VERIFIER};

use crate::util::sec_to_years;
use crate::verify::verify_signature;
use aurans_name::state::Metadata;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, PricesResponse, QueryMsg, VerifyMsg};

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
        operator: deps.api.addr_validate(&msg.operator)?,
        name_code_id: msg.name_code_id,
        resolver_code_id: msg.resolver_code_id,
        max_year_register: msg.max_year_register,
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
        msg: to_binary(&aurans_name::InstantiateMsg {
            admin: config.admin.to_string(),
            minter: env.contract.address.to_string(),
            resolver_code_id: config.resolver_code_id,
        })?,
        funds: vec![],
        label: "aurans-name".to_owned(),
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
        .add_attribute("max_year_register", msg.max_year_register.to_string())
        .add_attribute(
            "prices",
            msg.prices
                .iter()
                .map(|(l, price)| format!("{}:{}", l, price.to_string()))
                .collect::<Vec<String>>()
                .join(","),
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
    match msg {
        ExecuteMsg::UpdateConfig {
            admin,
            operator,
            name_code_id,
            resolver_code_id,
            max_year_register,
        } => execute_update_config(
            deps,
            env,
            info,
            admin,
            operator,
            name_code_id,
            resolver_code_id,
            max_year_register,
        ),
        ExecuteMsg::UpdatePrices { prices } => execute_update_prices(deps, env, info, prices),
        ExecuteMsg::UpdateVerifier { backend_pubkey } => {
            execute_update_verifier(deps, env, info, backend_pubkey)
        }
        ExecuteMsg::Register {
            name,
            backend_signature,
            metadata,
        } => execute_register(deps, env, info, name, backend_signature, metadata),
        ExecuteMsg::Extend {
            name,
            backend_signature,
            durations,
        } => execute_extend(deps, env, info, name, backend_signature, durations),
        ExecuteMsg::Unregister { names } => execute_unregister(deps, env, info, names),
        ExecuteMsg::Withdraw { receiver, coin } => {
            execute_withdraw(deps, env, info, receiver, coin)
        }
    }
}

fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: String,
    coin: Coin,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let receiver_addr = deps.api.addr_validate(&receiver)?;
    let withdraw_msg = BankMsg::Send {
        to_address: receiver_addr.to_string(),
        amount: vec![coin.clone()],
    };
    Ok(Response::new()
        .add_message(withdraw_msg)
        .add_attribute("action", "withdraw")
        .add_attribute("receiver", receiver)
        .add_attribute("amount", coin.to_string()))
}

fn execute_unregister(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    names: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.operator {
        return Err(ContractError::Unauthorized {});
    }

    let mut token_ids = Vec::new();
    for name in names.clone() {
        if !REGISTERS.has(deps.storage, &name) {
            return Err(ContractError::NameNotRegistered { name });
        }
        let _ = REGISTERS.load(deps.storage, &name)?;
        token_ids.push(name.clone());
        REGISTERS.remove(deps.storage, &name);
    }

    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    let burn_tokens_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Extension {
            msg: aurans_name::NameExecuteMsg::BurnTokens { token_ids },
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(burn_tokens_msg)
        .add_attribute("action", "unregister")
        .add_attribute("names", names.join(",")))
}

fn execute_extend(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    backend_signature: Binary,
    durations: u64,
) -> Result<Response, ContractError> {
    if !REGISTERS.has(deps.storage, &name) {
        return Err(ContractError::NameNotRegistered { name });
    }
    let old_expires = REGISTERS.load(deps.storage, &name)?;
    let config = CONFIG.load(deps.storage)?;

    let years = sec_to_years(durations);
    if years == 0 {
        return Err(ContractError::InvalidDurations);
    }
    if years > config.max_year_register as u64 {
        return Err(ContractError::LimitYearRegister);
    }

    // Check user funds
    let fee = calc_price(deps.as_ref(), &name, &years)?;
    check_fee(fee, &info.funds)?;

    // If not owner, check verification msg
    if config.admin != info.sender {
        let verify_msg = VerifyMsg::Extend {
            name: name.clone(),
            sender: info.sender.to_string(),
            chain_id: env.block.chain_id,
            durations,
        };
        let verify_msg_str =
            serde_json_wasm::to_string(&verify_msg).map_err(|_| ContractError::SerdeError)?;

        let verifier = VERIFIER.load(deps.storage)?;

        verify_signature(
            deps.as_ref(),
            &verify_msg_str,
            &backend_signature.as_slice(),
            &verifier.backend_pubkey,
        )?;
    }
    // Get name contract
    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    let new_expires = old_expires + durations;

    let old_token: NftInfoResponse<Metadata> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: name_contract.to_string(),
            msg: to_binary(&aurans_name::QueryMsg::NftInfo {
                token_id: name.clone(),
            })?,
        }))?;

    // Burn old name
    let burn_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Burn {
            token_id: name.clone(),
        })?,
        funds: vec![],
    };

    let mut extension = old_token.extension;
    extension.durations = durations;

    // Mint new name
    let mint_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Mint {
            token_id: name.clone(),
            owner: info.sender.clone().to_string(),
            token_uri: old_token.token_uri,
            extension,
        })?,
        funds: vec![],
    };

    REGISTERS.save(deps.storage, &name, &new_expires)?;

    Ok(Response::new()
        .add_message(burn_msg)
        .add_message(mint_msg)
        .add_attribute("action", "extend")
        .add_attribute("sender", info.sender)
        .add_attribute("name", name)
        .add_attribute("new_expires", new_expires.to_string())
        .add_attribute("durations", durations.to_string()))
}

fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    backend_signature: Binary,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    // Check name is registed or not
    if REGISTERS.has(deps.storage, &name) {
        return Err(ContractError::NameRegistered { name });
    }

    let config = CONFIG.load(deps.storage)?;
    let durations = metadata.durations;
    let years = sec_to_years(durations);
    if years == 0 {
        return Err(ContractError::InvalidDurations);
    }
    if years > config.max_year_register {
        return Err(ContractError::LimitYearRegister);
    }

    // Check fee
    let fee = calc_price(deps.as_ref(), &name, &years)?;
    check_fee(fee, &info.funds)?;

    let bech32_prefixes = metadata.bech32_prefixes;

    // If not owner, check verification msg
    if config.admin != info.sender {
        let verify_msg = VerifyMsg::Register {
            name: name.clone(),
            sender: info.sender.to_string(),
            chain_id: env.block.chain_id,
            bech32_prefixes: bech32_prefixes.clone(),
            durations,
        };
        let verify_msg_str =
            serde_json_wasm::to_string(&verify_msg).map_err(|_| ContractError::SerdeError)?;

        let verifier = VERIFIER.load(deps.storage)?;

        verify_signature(
            deps.as_ref(),
            &verify_msg_str,
            &backend_signature.as_slice(),
            &verifier.backend_pubkey,
        )?;
    }

    let register_secs = env.block.time.seconds();
    let expires_secs = register_secs + durations;

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
                durations,
                collection_name: metadata.collection_name,
                collection_symbol: metadata.collection_symbol,
            },
        })?,
        funds: vec![],
    };

    REGISTERS.save(deps.storage, &name, &expires_secs)?;

    Ok(Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "register")
        .add_attribute("sender", info.sender)
        .add_attribute("name", name)
        .add_attribute("bech32_prefixes", bech32_prefixes.join(","))
        .add_attribute("durations", durations.to_string())
        .add_attribute("register_at", register_secs.to_string())
        .add_attribute("expires_at", expires_secs.to_string())
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
                .join(","),
        ))
}

fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
    operator: String,
    name_code_id: u64,
    resolver_code_id: u64,
    max_year_register: u64,
) -> Result<Response, ContractError> {
    // only contract admin can update config
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update config
    let new_config = Config {
        admin: deps.api.addr_validate(&admin)?,
        operator: deps.api.addr_validate(&operator)?,
        name_code_id,
        resolver_code_id,
        max_year_register,
    };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", admin.to_string())
        .add_attribute("operator", operator.to_string())
        .add_attribute("name_code_id", name_code_id.to_string())
        .add_attribute("resolver_code_id", resolver_code_id.to_string())
        .add_attribute("max_year_register", max_year_register.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Verifier {} => to_binary(&query_verifier(deps)?),
        QueryMsg::Prices {} => to_binary(&query_prices(deps)?),
        QueryMsg::NameContract {} => to_binary(&query_name_contract(deps)?),
        QueryMsg::HasRegister { name } => to_binary(&query_has_register(deps, name)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_verifier(deps: Deps) -> StdResult<Verifier> {
    VERIFIER.load(deps.storage)
}

fn query_prices(deps: Deps) -> StdResult<PricesResponse> {
    let prices_res: StdResult<Vec<_>> = PRICE_INFO
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    match prices_res {
        Ok(prices) => Ok(PricesResponse { prices: prices }),
        Err(_) => Ok(PricesResponse { prices: vec![] }),
    }
}

fn query_name_contract(deps: Deps) -> StdResult<Addr> {
    NAME_CONTRACT.load(deps.storage)
}

fn query_has_register(deps: Deps, name: String) -> StdResult<bool> {
    Ok(REGISTERS.has(deps.storage, &name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let name_contract = deps.api.addr_validate(&reply.contract_address)?;
    NAME_CONTRACT.save(deps.storage, &name_contract)?;

    Ok(Response::new().add_attribute("name_contract", name_contract))
}
