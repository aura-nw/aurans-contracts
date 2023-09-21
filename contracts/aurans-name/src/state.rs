use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Metadata {
    pub bech32_prefix_registed: Vec<String>,
    pub ttl: u64,
    pub is_expried: Option<bool>,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub resolver_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
