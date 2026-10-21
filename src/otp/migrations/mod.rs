use color_eyre::eyre::eyre;

use super::otp_element::{CURRENT_DATABASE_VERSION, OTPDatabase};

/// Upgrade legacy vaults without changing account credentials. IDs are also repaired
/// after imports, which can contain missing or duplicate identifiers.
pub fn migrate(database: &mut OTPDatabase) -> color_eyre::Result<()> {
    if database.version > CURRENT_DATABASE_VERSION {
        return Err(eyre!(
            "This vault uses a newer database format; upgrade cotp"
        ));
    }
    let changed = database.version < CURRENT_DATABASE_VERSION;
    database.ensure_account_ids()?;
    database.version = CURRENT_DATABASE_VERSION;
    if changed {
        database.mark_modified();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::otp_element::OTPElementBuilder;

    #[test]
    fn legacy_accounts_keep_secrets_and_receive_persistent_unique_ids() {
        let element = OTPElementBuilder::default()
            .secret("JBSWY3DPEHPK3PXP")
            .issuer("Example")
            .label("alice")
            .counter(Some(17))
            .build()
            .unwrap();
        let mut database = OTPDatabase::from(vec![element.clone(), element]);
        migrate(&mut database).unwrap();
        assert!(database.is_modified());
        assert_eq!(database.version, CURRENT_DATABASE_VERSION);
        assert_ne!(database.elements[0].id, database.elements[1].id);
        assert_eq!(database.elements[0].secret, "JBSWY3DPEHPK3PXP");
        assert_eq!(database.elements[0].counter, Some(17));
        let serialized = serde_json::to_string(&database).unwrap();
        let mut restored: OTPDatabase = serde_json::from_str(&serialized).unwrap();
        migrate(&mut restored).unwrap();
        assert!(!restored.is_modified());
        assert_eq!(database.elements, restored.elements);
    }

    #[test]
    fn version_two_without_metadata_deserializes() {
        let mut database: OTPDatabase = serde_json::from_str(r#"{"version":2,"elements":[{"secret":"AA","issuer":"Old","label":"account","digits":6,"type":"TOTP","algorithm":"SHA1","period":30,"counter":null,"pin":null}]}"#).unwrap();
        migrate(&mut database).unwrap();
        assert_eq!(database.elements[0].id.len(), 32);
        assert!(database.elements[0].tags.is_empty());
        assert!(database.elements[0].alias.is_none());
    }

    #[test]
    fn imported_duplicate_ids_are_repaired_without_changing_existing_id() {
        let element = OTPElementBuilder::default()
            .id("stable-id")
            .secret("AA")
            .issuer("Example")
            .label("alice")
            .build()
            .unwrap();
        let mut database = OTPDatabase::from(vec![element.clone(), element]);
        migrate(&mut database).unwrap();
        assert_eq!(database.elements[0].id, "stable-id");
        assert_ne!(database.elements[1].id, "stable-id");
    }

    #[test]
    fn newer_database_format_is_rejected() {
        let mut database = OTPDatabase {
            version: CURRENT_DATABASE_VERSION + 1,
            ..Default::default()
        };
        assert!(migrate(&mut database).is_err());
    }
}
