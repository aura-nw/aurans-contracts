use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    // Address of name contract
    pub name_contract: Addr,
}

pub struct RecordIndexes<'a> {
    pub address: MultiIndex<'a, String, String, String>,
}

impl<'a> IndexList<String> for RecordIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<String>> + '_> {
        let v: Vec<&dyn Index<String>> = vec![&self.address];
        Box::new(v.into_iter())
    }
}

// A IndexedMap map a tuple of name record to address: (primary_name, bech32_prefix) -> address
// Example: ("tiennv", "aura") -> "aura12aabc..."
pub fn records<'a>() -> IndexedMap<'a, (&'a str, &'a str), String, RecordIndexes<'a>> {
    let indexes = RecordIndexes {
        address: MultiIndex::new(
            |_pk, addr: &String| addr.clone(),
            "records",
            "records__address",
        ),
    };
    IndexedMap::new("records", indexes)
}

pub const CONFIG: Item<Config> = Item::new("config");
