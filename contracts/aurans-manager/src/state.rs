use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,

    pub name_code_id: u64,
    pub resolver_code_id: u64,

    pub max_year_register: u64,
}

#[cw_serde]
pub struct Verifier {
    pub backend_pubkey: Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");
// PRICE_INFO has keys are length of name.
// The value of key is zero meaning other length of name not in config
pub const PRICE_INFO: Map<u8, Coin> = Map::new("price_info");
pub const VERIFIER: Item<Verifier> = Item::new("verify");
pub const NAME_CONTRACT: Item<Addr> = Item::new("name_contract");
// A map name registed with expires (seconds)
pub const REGISTERS: Map<&str, u64> = Map::new("registers");
