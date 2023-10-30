use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    // Extend from cw2981-royalties
    // see: https://docs.opensea.io/docs/metadata-standards
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// This is how much the minter takes as a cut when sold
    /// royalties are owed on this token if it is Some
    pub royalty_percentage: Option<u64>,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: Option<String>,

    // List bech32 prefix register
    pub bech32_prefixes: Vec<String>,

    // Lifetime duration of nft in seconds
    pub expires: u64,
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub minter: Addr,
}

#[cw_serde]
pub struct Resolver {
    pub address: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RESOLVER: Item<Resolver> = Item::new("resolver");
