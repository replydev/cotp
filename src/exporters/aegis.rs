//! Aegis v1 container / v3 database with an scrypt password slot and AES-256-GCM.
use crate::otp::{otp_element::OTPDatabase, otp_type::OTPType};
use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, Nonce},
};
use color_eyre::eyre::{Result, eyre};
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub fn encode(database: &OTPDatabase, password: Option<&str>) -> Result<Value> {
    let mut entries = Vec::new();
    let mut groups = Vec::<Value>::new();
    let mut group_ids = std::collections::BTreeMap::<String, String>::new();
    for element in database.elements_ref() {
        if !matches!(
            element.type_,
            OTPType::Totp | OTPType::Hotp | OTPType::Steam
        ) {
            return Err(eyre!(
                "Aegis export supports TOTP, HOTP, and Steam accounts only"
            ));
        }
        let mut entry_groups = Vec::new();
        for tag in &element.tags {
            let id = if let Some(id) = group_ids.get(tag) {
                id.clone()
            } else {
                let id = uuid()?;
                group_ids.insert(tag.clone(), id.clone());
                groups.push(json!({"uuid":id,"name":tag}));
                id
            };
            entry_groups.push(id);
        }
        entries.push(json!({"type":element.type_.to_string().to_lowercase(),"uuid":uuid()?,"name":element.label,"issuer":element.issuer,"note":"","icon":null,"groups":entry_groups,"favorite":element.favorite,"info":{"secret":element.secret,"algo":element.algorithm.to_string().to_uppercase(),"digits":element.digits,"period":element.period,"counter":element.counter}}));
    }
    let db = json!({"version":3,"entries":entries,"groups":groups,"icons_optimized":true});
    let Some(password) = password else {
        return Ok(json!({"version":1,"header":{"slots":null,"params":null},"db":db}));
    };
    if password.is_empty() {
        return Err(eyre!("Encrypted export requires a nonempty password"));
    }
    let plaintext = Zeroizing::new(serde_json::to_vec(&db)?);
    let master = Zeroizing::new(random::<32>()?);
    let salt = random::<32>()?;
    let mut derived = Zeroizing::new([0u8; 32]);
    let params = scrypt::Params::new(15, 8, 1).map_err(|_| eyre!("Invalid scrypt parameters"))?;
    scrypt::scrypt(password.as_bytes(), &salt, &params, derived.as_mut())
        .map_err(|_| eyre!("Scrypt failed"))?;
    let (key, key_params) = encrypt(derived.as_slice(), master.as_slice())?;
    let (encrypted, crypt_params) = encrypt(master.as_slice(), &plaintext)?;
    Ok(
        json!({"version":1,"header":{"slots":[{"type":1,"uuid":uuid()?,"key":hex::encode(key),"key_params":key_params,"n":32768,"r":8,"p":1,"salt":hex::encode(salt),"repaired":true,"is_backup":true}],"params":crypt_params},"db":data_encoding::BASE64.encode(&encrypted)}),
    )
}
fn random<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).map_err(|_| eyre!("OS random generation failed"))?;
    Ok(bytes)
}
fn uuid() -> Result<String> {
    let mut bytes = random::<16>()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let s = hex::encode(bytes);
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &s[..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..]
    ))
}
fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Value)> {
    let nonce = random::<12>()?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| eyre!("Invalid AES key"))?;
    let nonce_value =
        Nonce::<Aes256Gcm>::try_from(nonce.as_slice()).map_err(|_| eyre!("Invalid nonce"))?;
    let mut ciphertext = cipher
        .encrypt(&nonce_value, plaintext)
        .map_err(|_| eyre!("Encryption failed"))?;
    let tag = ciphertext.split_off(ciphertext.len() - 16);
    Ok((
        ciphertext,
        json!({"nonce":hex::encode(nonce),"tag":hex::encode(tag)}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::importers::aegis_encrypted::AegisEncryptedDatabase;
    use crate::otp::{from_otp_uri::FromOtpUri, otp_element::OTPElement};
    #[test]
    fn encrypted_backup_round_trip_and_wrong_password() {
        let mut element =
            OTPElement::from_otp_uri("otpauth://hotp/Example:alice?secret=JBSWY3DP&counter=99")
                .unwrap();
        element.favorite = true;
        element.tags = vec!["work".into()];
        let mut database = OTPDatabase::default();
        database.add_element(element).unwrap();
        let encoded = encode(&database, Some("correct horse battery staple")).unwrap();
        assert!(
            !serde_json::to_string(&encoded)
                .unwrap()
                .contains("JBSWY3DP")
        );
        let input: AegisEncryptedDatabase = serde_json::from_value(encoded.clone()).unwrap();
        let restored = input
            .decrypt_with_password("correct horse battery staple")
            .unwrap();
        assert_eq!(restored[0].secret, "JBSWY3DP");
        assert_eq!(restored[0].counter, Some(99));
        assert!(restored[0].favorite);
        assert_eq!(restored[0].tags, ["work"]);
        let plain = encode(&database, None).unwrap();
        let restored = crate::importers::enrollment::parse_json(plain, "aegis").unwrap();
        assert_eq!(restored.elements[0].tags, ["work"]);
        let input: AegisEncryptedDatabase = serde_json::from_value(encoded).unwrap();
        assert!(input.decrypt_with_password("wrong").is_err());
    }
}
