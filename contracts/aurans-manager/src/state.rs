use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin, Timestamp};
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
// PRICE_INFO has keys are length of name.
// The value of key is zero meaning other length of name not in config
pub const PRICE_INFO: Map<u8, Coin> = Map::new("price_info");
pub const VERIFIER: Item<Verifier> = Item::new("verify");
pub const NAME_CONTRACT: Item<Addr> = Item::new("name_contract");

// 365 days
pub const SEC_PER_YEAR: u64 = 31536000;

pub fn years_from_expires(old: &Timestamp, new: &Timestamp) -> u64 {
    new.minus_seconds(old.seconds()).seconds() / SEC_PER_YEAR
}

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::Timestamp;

    use super::years_from_expires;

    #[test]
    fn test_years_from_expires() {
        // Date and time (GMT): Friday, October 13, 2023 12:00:00 AM
        let old = Timestamp::from_seconds(1697155200);
        // Date and time (GMT): Friday, October 13, 2024 12:00:00 AM
        let new = Timestamp::from_seconds(1728777600);
        let years = years_from_expires(&old, &new);
        assert_eq!(years, 1);
    }
}
