use crate::error::ContractError;
use cosmwasm_std::Deps;
use sha2::Digest;

pub fn verify_signature(
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

#[cfg(test)]
pub mod tests {
    use cosmrs::{bip32, crypto::secp256k1::SigningKey, tendermint::crypto::Sha256};
    use cosmwasm_crypto::secp256k1_verify;
    use cosmwasm_std::Binary;

    use crate::{msg::VerifyMsg, util::year_to_secs};

    fn from_mnemonic(phrase: &str, derivation_path: &str) -> SigningKey {
        let seed = bip32::Mnemonic::new(phrase, bip32::Language::English)
            .unwrap()
            .to_seed("");
        let xprv = bip32::XPrv::derive_from_path(seed, &derivation_path.parse().unwrap()).unwrap();
        xprv.into()
    }

    #[test]
    fn test_basic() {
        let derivation_path = "m/44'/118'/0'/0/0";
        let mnemonic = "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius";

        let verifier = from_mnemonic(mnemonic, derivation_path);

        let binary = Binary(verifier.public_key().to_bytes());
        let pubkey_hex = binary.to_string();
        println!("public_key: {:?}", &pubkey_hex);

        let binary_again = Binary::from_base64(&pubkey_hex).unwrap();

        assert_eq!(binary, binary_again);

        let one_year = year_to_secs(1);
        println!("durations={:?}", one_year);

        let register_msg = VerifyMsg::Register {
            name: "tiennv".to_owned(),
            sender: "aura1yntfxtwysmgjp6wzza590xctjpzne3ak9scynv".to_owned(),
            chain_id: "aura-local".to_owned(),
            bech32_prefixes: vec!["aura".to_owned(), "cosmos".to_owned()],
            durations: one_year,
        };

        let register_msg_json = serde_json_wasm::to_string(&register_msg).unwrap();

        let register_msg_hash = sha2::Sha256::digest(register_msg_json.clone());

        let sig = verifier
            .sign(register_msg_json.as_bytes())
            .unwrap()
            .to_vec();
        let sig_binary = Binary(sig.clone());
        println!("sig={:?}", sig_binary.to_string());

        let verified =
            secp256k1_verify(&register_msg_hash, &sig, &verifier.public_key().to_bytes()).unwrap();
        assert!(verified);

        let extend_msg = VerifyMsg::Extend {
            name: "tiennv".to_owned(),
            sender: "aura1yntfxtwysmgjp6wzza590xctjpzne3ak9scynv".to_string(),
            chain_id: "aura-local".to_owned(),
            durations: one_year,
        };

        let extend_msg_json = serde_json_wasm::to_string(&extend_msg).unwrap();
        let sig = verifier.sign(extend_msg_json.as_bytes()).unwrap().to_vec();
        let sig_binary = Binary(sig.clone());
        println!("sig={:?}", sig_binary.to_string());
    }
}
