use crate::cli::Cli;
use config::{Config, ConfigError, Environment, File};
use directories::ProjectDirs;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub application_name: String,
    pub fabrex_base_url: String,
    pub gryf_base_url: String,
    pub supernode_base_url: String,
    pub poll_interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            application_name: "FabreXLens".to_string(),
            fabrex_base_url: "https://api.gigaio.com/fabrexfleet".to_string(),
            gryf_base_url: "https://api.gigaio.com/gryf".to_string(),
            supernode_base_url: "https://api.gigaio.com/supernodes".to_string(),
            poll_interval_secs: 15,
        }
    }
}

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
}

impl AppConfig {
    pub fn load(cli: &Cli) -> Result<Self, AppConfigError> {
        let defaults = Self::default();
        let mut builder = Config::builder()
            .set_default("application_name", defaults.application_name.clone())?
            .set_default("fabrex_base_url", defaults.fabrex_base_url.clone())?
            .set_default("gryf_base_url", defaults.gryf_base_url.clone())?
            .set_default(
                "supernode_base_url",
                defaults.supernode_base_url.clone(),
            )?
            .set_default("poll_interval_secs", defaults.poll_interval_secs)?;

        if let Some(profile) = &cli.profile {
            let profile_file_name = format!("fabrexlens.{profile}.toml");
            if let Some(path) = Self::profile_path(&profile_file_name) {
                builder = builder.add_source(File::from(path).required(false));
            }
        }

        if let Some(config_path) = &cli.config {
            builder = builder.add_source(File::from(config_path.clone()).required(true));
        } else if let Some(path) = Self::default_config_path() {
            builder = builder.add_source(File::from(path).required(false));
        }

        builder = builder.add_source(Environment::with_prefix("FABREXLENS").separator("__"));

        let built = builder.build()?;
        Ok(built.try_deserialize::<AppConfig>()?)
    }

    fn default_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "DigitalDataCo", "FabreXLens")
            .map(|dirs| dirs.config_dir().join("fabrexlens.toml"))
    }

    fn profile_path(file_name: &str) -> Option<PathBuf> {
        ProjectDirs::from("com", "DigitalDataCo", "FabreXLens")
            .map(|dirs| dirs.config_dir().join(file_name))
    }
}

