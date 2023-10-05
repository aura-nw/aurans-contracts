use cosmwasm_std::{Coin, Storage};

use crate::{
    error::ContractError,
    state::{NameLen, PRICE_INFO},
};

pub fn calc_price(storage: &dyn Storage, name: &str) -> Result<Coin, ContractError> {
    let name_len = NameLen::from_len(&name.len());
    let price = PRICE_INFO.load(storage, &name_len.to_string())?;
    Ok(price)
}

pub fn check_fee(price: Coin, funds: &Vec<Coin>) -> Result<(), ContractError> {
    if funds
        .into_iter()
        .any(|fund| fund.denom == price.denom && fund.amount > price.amount)
    {
        Ok(())
    } else {
        Err(ContractError::InsufficientFunds)
    }
}
