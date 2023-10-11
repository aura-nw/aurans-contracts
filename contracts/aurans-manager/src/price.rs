use cosmwasm_std::{Coin, Deps};

use crate::{error::ContractError, state::PRICE_INFO};

pub fn calc_price(deps: Deps, name: &str) -> Result<Coin, ContractError> {
    let name_len = name.len() as u8;
    let price = PRICE_INFO.load(deps.storage, name_len)?;
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
