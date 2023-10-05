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

#[cw_serde]
pub enum NameLen {
    One,
    Two,
    Three,
    Four,
    Other,
}

impl NameLen {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "one" => Some(Self::One),
            "two" => Some(Self::Two),
            "three" => Some(Self::Three),
            "four" => Some(Self::Four),
            "other" => Some(Self::Other),
            _ => None,
        }
    }

    pub fn from_len(s: &usize) -> Self {
        match s {
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            _ => Self::Other,
        }
    }
}

impl std::fmt::Display for NameLen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            NameLen::One => "One",
            NameLen::Two => "Two",
            NameLen::Three => "Three",
            NameLen::Four => "Four",
            NameLen::Other => "Other",
        };
        write!(f, "{}", name)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PRICE_INFO: Map<&str, Coin> = Map::new("price_info");
pub const VERIFIER: Item<Verifier> = Item::new("verify");
pub const NAME_CONTRACT: Item<Addr> = Item::new("name_contract");
