mod app;
mod cli;
mod config;
mod services;
mod ui;

use crate::cli::Command;
use crate::services::auth::{CredentialKey, CredentialManager};
use anyhow::Result;
use clap::Parser;
use std::panic;

fn main() -> Result<()> {
    panic::set_hook(Box::new(|info| {
        eprintln!("FabreXLens panic: {info}");
        if let Some(location) = info.location() {
            eprintln!(
                "  at {}:{}",
                location.file(),
                location.line()
            );
        }
    }));

    let cli = cli::Cli::parse();

    if let Some(command) = cli.command.clone() {
        return handle_command(command);
    }

    let settings = config::AppConfig::load(&cli)?;

    if cli.headless {
        println!("Headless mode is not yet available. Launching UI skipped.");
        return Ok(());
    }

    app::run(settings)
}

fn handle_command(command: Command) -> Result<()> {
    match command {
        Command::AuthInit { domain, scope } => {
            let manager = CredentialManager::with_default_keyring();
            let key = CredentialKey::new(domain.into(), scope);
            let secret = manager.ensure_credentials(&key)?;
            println!(
                "Credentials stored for {} ({})",
                key,
                secret.redacted_summary()
            );
        }
    }
    Ok(())
}

