use crate::services::auth::CredentialDomain;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Command line interface for FabreXLens.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "FabreXLens",
    author,
    version,
    about = "Interactive observability tool for GigaIO fabrics"
)]
pub struct Cli {
    /// Optional path to a configuration file (TOML, YAML, JSON).
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Named profile to load (e.g. dev, staging, prod).
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Launch without opening the UI (useful for scripting and diagnostics).
    #[arg(long)]
    pub headless: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Capture credentials for a specific GigaIO service.
    AuthInit {
        #[arg(value_enum)]
        domain: CredentialDomainArg,
        #[arg(short, long, default_value = "default")]
        scope: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CredentialDomainArg {
    FabreX,
    Gryf,
    Supernode,
    Redfish,
}

impl From<CredentialDomainArg> for CredentialDomain {
    fn from(value: CredentialDomainArg) -> Self {
        match value {
            CredentialDomainArg::FabreX => CredentialDomain::FabreX,
            CredentialDomainArg::Gryf => CredentialDomain::Gryf,
            CredentialDomainArg::Supernode => CredentialDomain::Supernode,
            CredentialDomainArg::Redfish => CredentialDomain::Redfish,
        }
    }
}
