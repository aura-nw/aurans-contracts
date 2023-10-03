use std::ops::Mul;

use cosmwasm_std::{Coin, Uint128};

use crate::error::ContractError;

pub fn calc_price(name: &str, _prefix: &str, base_price: Coin) -> Coin {
    if name.len() <= 3 {
        return Coin {
            denom: base_price.denom,
            amount: base_price.amount.mul(Uint128::from(10u64)),
        };
    }
    if name.len() == 4 {
        return Coin {
            denom: base_price.denom,
            amount: base_price.amount.mul(Uint128::from(5u64)),
        };
    }

    return base_price;
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

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{Coin, Uint128};

    use super::calc_price;

    #[test]
    fn test_price() {
        let base_price = Coin {
            denom: String::from("aura"),
            amount: Uint128::from(10u64),
        };

        let n3 = "aaa";
        let pf3 = "aura";

        let n4 = "aaba";
        let pf4 = "cosmos";

        let real_price = calc_price(n3, pf3, base_price.clone());
        assert!(real_price.amount == Uint128::from(100u64));

        let real_price = calc_price(n4, pf4, base_price.clone());
        assert!(real_price.amount == Uint128::from(50u64));
    }
}
