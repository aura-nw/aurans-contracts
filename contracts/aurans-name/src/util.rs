use crate::error::ContractError;

// Extract a name from given token_id.
// token_id has format: name@[timestamp in secs]
pub fn extract_name_from_token_id(token_id: &str) -> Result<(&str, u64), ContractError> {
    let parts: Vec<&str> = token_id.split("@").collect();
    if parts.len() != 2 {
        Err(ContractError::InvalidTokenId {})
    } else {
        let expires = parts[1]
            .parse::<u64>()
            .map_err(|_| ContractError::InvalidTokenId {})?;
        Ok((parts[0], expires))
    }
}

pub fn join_name_and_expires(name: &str, expires: u64) -> String {
    format!("{}@{}", name, expires)
}

#[cfg(test)]
mod tests {

    use crate::util::join_name_and_expires;

    use super::extract_name_from_token_id;

    #[test]
    fn test_extract_ok() {
        let token_id = "tiennv@100000000000";
        let name = extract_name_from_token_id(token_id);
        assert!(name.is_ok());
        let (name, expires) = name.unwrap();
        assert_eq!(name, "tiennv");
        assert_eq!(expires, 100000000000);
    }

    #[test]
    fn test_join_name_and_expires() {
        let name = "tiennv";
        let expires = 1729331450;
        assert_eq!(join_name_and_expires(name, expires), "tiennv@1729331450");
    }
}
