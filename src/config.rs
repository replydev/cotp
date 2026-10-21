use crate::arguments::CotpArgs;
use color_eyre::eyre::{WrapErr, eyre};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf, sync::OnceLock};

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub database_path: Option<PathBuf>,
    pub profiles: BTreeMap<String, PathBuf>,
    pub default_profile: Option<String>,
    pub idle_timeout_seconds: u64,
    pub clipboard_seconds: u64,
    pub hide_codes: bool,
    pub keychain: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_path: None,
            profiles: BTreeMap::new(),
            default_profile: None,
            idle_timeout_seconds: 300,
            clipboard_seconds: 30,
            hide_codes: false,
            keychain: false,
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(Settings::default)
}

pub fn load(args: &mut CotpArgs) -> color_eyre::Result<()> {
    let explicit = args
        .config
        .clone()
        .or_else(|| std::env::var_os("COTP_CONFIG").map(PathBuf::from));
    let path = explicit
        .clone()
        .or_else(|| dirs::config_dir().map(|p| p.join("cotp/config.json")));
    let config: Settings = match path {
        Some(path) => match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .wrap_err_with(|| format!("Invalid configuration: {}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && explicit.is_none() => {
                Settings::default()
            }
            Err(e) => {
                return Err(e)
                    .wrap_err_with(|| format!("Cannot read configuration: {}", path.display()));
            }
        },
        None => Settings::default(),
    };
    let profile = args
        .profile
        .clone()
        .or_else(|| std::env::var("COTP_PROFILE").ok())
        .or_else(|| config.default_profile.clone());
    let configured_path = profile
        .as_ref()
        .map(|name| {
            config
                .profiles
                .get(name)
                .cloned()
                .ok_or_else(|| eyre!("Unknown vault profile: {name}"))
        })
        .transpose()?
        .or_else(|| config.database_path.clone());
    if args.database_path.is_none() && std::env::var_os("COTP_DB_PATH").is_none() {
        args.database_path = configured_path.map(|p| p.to_string_lossy().into_owned());
    }
    SETTINGS
        .set(config)
        .map_err(|_| eyre!("Configuration already loaded"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_validates_names_and_preserves_defaults() {
        let config: Settings =
            serde_json::from_str(r#"{"profiles":{"work":"/vault/work.cotp"},"hide_codes":true}"#)
                .unwrap();
        assert_eq!(config.profiles["work"], PathBuf::from("/vault/work.cotp"));
        assert_eq!(config.idle_timeout_seconds, 300);
        assert!(config.hide_codes);
        assert!(serde_json::from_str::<Settings>(r#"{"hid_codes":true}"#).is_err());
    }
}
