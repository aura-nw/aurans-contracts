use crate::error::ContractError;

// Extract a name from given token_id.
// token_id has format: name@[timestamp in secs]
pub fn extract_name_from_token_id(token_id: &str) -> Result<&str, ContractError> {
    let parts: Vec<&str> = token_id.split("@").collect();
    if parts.len() != 2 {
        Err(ContractError::InvalidTokenId {})
    } else {
        Ok(parts[0])
    }
}

#[cfg(test)]
mod tests {

    use super::extract_name_from_token_id;

    #[test]
    fn test_extract_ok() {
        let token_id = "tiennv@100000000000";
        let name = extract_name_from_token_id(token_id);
        assert!(name.is_ok());
    }
}
