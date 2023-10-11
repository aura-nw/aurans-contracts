use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,

    pub name_code_id: u64,
    pub resolver_code_id: u64,
}

#[cw_serde]
pub struct Verifier {
    pub backend_pubkey: Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PRICE_INFO: Map<u8, Coin> = Map::new("price_info");
pub const VERIFIER: Item<Verifier> = Item::new("verify");
pub const NAME_CONTRACT: Item<Addr> = Item::new("name_contract");
