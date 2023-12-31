use std::ops::Mul;

use cosmwasm_std::{Coin, Deps, Uint128};

use crate::{error::ContractError, state::PRICE_INFO};

pub fn calc_price(deps: Deps, name: &str, years: &u64) -> Result<Coin, ContractError> {
    let name_len = name.len() as u8;
    let has_price = PRICE_INFO.has(deps.storage, name_len);
    if !has_price {
        // Base price
        let price = PRICE_INFO.load(deps.storage, 0)?;
        return Ok(price);
    }
    let mut price = PRICE_INFO.load(deps.storage, name_len)?;
    price.amount = price.amount.mul(Uint128::from(*years));
    Ok(price)
}

pub fn check_fee(price: Coin, funds: &Vec<Coin>) -> Result<(), ContractError> {
    if funds
        .into_iter()
        .any(|fund| fund.denom == price.denom && fund.amount >= price.amount)
    {
        Ok(())
    } else {
        Err(ContractError::InsufficientFunds)
    }
}
