use subtle_encoding::bech32;

use crate::error::ContractError;

pub fn bech32_decode(addr: &str) -> Result<Vec<u8>, ContractError> {
    let (_, data) = bech32::decode(addr).map_err(|_| ContractError::Bech32DecodeError {})?;
    Ok(data)
}

pub fn bech32_encode(bech32_prefix: &str, data: &[u8]) -> String {
    bech32::encode(bech32_prefix, data)
}

#[cfg(test)]
pub mod tests {
    use crate::util::{bech32_decode, bech32_encode};

    #[test]
    fn test_bech32_encoding() {
        // juno1qcjgq3vqpgrjvmk2z9pcrv67f89ecayhyamsu9
        let aura_addr = "aura1qcjgq3vqpgrjvmk2z9pcrv67f89ecayhfe0feq";
        let aura_addr_decoded = bech32_decode(aura_addr).unwrap();

        let x = bech32_encode("juno", aura_addr_decoded.as_slice());
        assert_eq!(x, "juno1qcjgq3vqpgrjvmk2z9pcrv67f89ecayhyamsu9");
    }
}
