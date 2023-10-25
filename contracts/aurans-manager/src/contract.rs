#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, QueryRequest,
    Reply, ReplyOn, Response, StdResult, SubMsg, Timestamp, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::NftInfoResponse;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::price::{calc_price, check_fee};
use crate::state::{
    years_from_expires, Config, Verifier, CONFIG, NAME_CONTRACT, PRICE_INFO, VERIFIER,
};

use crate::verify::verify_signature;
use aurans_name::state::Metadata;
use aurans_name::util::join_name_and_expires;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:aurans-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_YEAR_REGISTER: u8 = 5;

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
        msg: to_binary(&aurans_name::InstantiateMsg {
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
            backend_signature,
            metadata,
        } => execute_register(deps, env, info, name, backend_signature, metadata),
        ExecuteMsg::ExtendExpires {
            name,
            backend_signature,
            old_expires,
            new_expires,
        } => execute_extend_expires(
            deps,
            env,
            info,
            name,
            backend_signature,
            old_expires,
            new_expires,
        ),
    }
}

fn execute_extend_expires(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    backend_signature: Binary,
    old_expires: u64,
    new_expires: u64,
) -> Result<Response, ContractError> {
    let years = years_from_expires(
        &Timestamp::from_seconds(old_expires),
        &Timestamp::from_seconds(new_expires),
    );
    if years > MAX_YEAR_REGISTER as u64 {
        return Err(ContractError::InvalidYearRegister);
    }

    // Check user funds
    let fee = calc_price(deps.as_ref(), &name, &(years as u8))?;
    check_fee(fee, &info.funds)?;

    let config = CONFIG.load(deps.storage)?;
    // If not owner, check verification msg
    if config.admin != info.sender {
        let verify_msg = VerifyMsg::ExtendExpires {
            name: name.clone(),
            sender: info.sender.to_string(),
            chain_id: env.block.chain_id,
            old_expires: old_expires,
            new_expires: new_expires,
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
    let old_token_id = join_name_and_expires(&name, old_expires);
    let new_token_id = join_name_and_expires(&name, new_expires);

    let old_token: NftInfoResponse<Metadata> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: name_contract.to_string(),
            msg: to_binary(&aurans_name::QueryMsg::NftInfo {
                token_id: old_token_id.clone(),
            })?,
        }))?;

    // Burn old name
    let burn_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Burn {
            token_id: old_token_id,
        })?,
        funds: vec![],
    };

    // Mint new name
    let mint_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Mint {
            token_id: new_token_id,
            owner: info.sender.clone().to_string(),
            token_uri: old_token.token_uri.clone(),
            extension: old_token.extension.clone(),
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(burn_msg)
        .add_message(mint_msg)
        .add_attribute("action", "extend_expires")
        .add_attribute("name", name)
        .add_attribute("old_expires", old_expires.to_string())
        .add_attribute("new_expires", new_expires.to_string()))
}

fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    backend_signature: Binary,
    metadata: Metadata,
) -> Result<Response, ContractError> {
    let years = metadata.years;
    if years > MAX_YEAR_REGISTER {
        return Err(ContractError::InvalidYearRegister);
    }

    let expires = metadata.expires;
    if expires <= env.block.time.seconds() {
        return Err(ContractError::InvalidTimestamp {
            blocktime: env.block.time.seconds().to_string(),
        });
    }

    // Check fee
    let fee = calc_price(deps.as_ref(), &name, &years)?;
    check_fee(fee, &info.funds)?;

    let config = CONFIG.load(deps.storage)?;
    let bech32_prefixes = metadata.bech32_prefixes;
    let expires = metadata.expires;
    // If not owner, check verification msg
    if config.admin != info.sender {
        let verify_msg = VerifyMsg::Register {
            name: name.clone(),
            sender: info.sender.to_string(),
            chain_id: env.block.chain_id,
            bech32_prefixes: bech32_prefixes.clone(),
            expires: expires,
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

    // Call mint msg
    let name_contract = NAME_CONTRACT.load(deps.storage)?;
    let token_id = join_name_and_expires(&name, expires);
    let mint_msg = WasmMsg::Execute {
        contract_addr: name_contract.to_string(),
        msg: to_binary(&aurans_name::ExecuteMsg::Mint {
            token_id,
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
                expires: expires.clone(),
                years: years.clone(),
            },
        })?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "register")
        .add_attribute("name", name)
        .add_attribute("bech32_prefixes", bech32_prefixes.join(","))
        .add_attribute("years", years.to_string())
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
        QueryMsg::Verifier {} => to_binary(&query_verifier(deps)?),
        QueryMsg::Prices {} => to_binary(&query_prices(deps)?),
        QueryMsg::NameContract {} => to_binary(&query_name_contract(deps)?),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let name_contract = deps.api.addr_validate(&reply.contract_address)?;
    NAME_CONTRACT.save(deps.storage, &name_contract)?;

    Ok(Response::new().add_attribute("name_contract", name_contract))
}
