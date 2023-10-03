use crate::error::ContractError;
use cosmwasm_std::Deps;
use sha2::Digest;

pub fn check_verify_signature(
    deps: Deps,
    msg: &str,
    signature: &[u8],
    pubkey: &[u8],
) -> Result<bool, ContractError> {
    let msg_hash = sha2::Sha256::digest(msg);
    let ok = deps
        .api
        .secp256k1_verify(&msg_hash, signature, pubkey)
        .map_err(|_| ContractError::VerificationError)?;
    if !ok {
        return Err(ContractError::InvalidSignature);
    }
    Ok(true)
}
