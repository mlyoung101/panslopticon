// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use log::info;

use crate::{repo::GhRemoteRepo, types::PanslopConfig, update::reconsider};

pub mod analyser;
pub mod ingress;
pub mod repo;
pub mod types;
pub mod update;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Subcommand)]
enum Commands {
    /// Ingress from GitHub
    #[command()]
    GHIngress {
        /// Config TOML path
        config: PathBuf,
    },

    /// Finds "ham" - good, non AI data
    #[command()]
    HamIngress {
        /// Config TOML path
        config: PathBuf,
    },

    /// Analyse all previously ingressed data
    #[command()]
    Analyse {
        /// Config TOML path
        config: PathBuf,
    },

    /// Analyse a single repository (for debugging)
    #[command()]
    AnalyseOne {
        /// Config TOML path
        config: PathBuf,

        /// URL to repo to analyse
        repo: String,
    },

    /// Update existing repositories
    #[command()]
    Update {
        /// Config TOML path
        config: PathBuf,
    },

    /// Reconsiders data in the not_slop table; useful if the scoring algorithm has been updated
    #[command()]
    Reconsider {
        /// Config TOML path
        config: PathBuf,
    },

    /// Prints version information.
    #[command()]
    Version {},
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "panslopticon")]
#[command(
    about = format!("Panslopticon (c) 2026 Mel Young; MPL 2.0"),
)]
struct PanslopticonCli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args = PanslopticonCli::parse();
    let env = Env::new().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();
    color_eyre::install()?;

    // https://github.com/snapview/tokio-tungstenite/issues/339#issuecomment-2424668126
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install TLS provider");

    dotenvy::dotenv()?;
    let db_url = std::env::var("DATABASE_URL")?;

    match args.command {
        Commands::GHIngress { config } => ingress::ingress_gh(config, db_url).await?,
        Commands::Analyse { config } => analyser::analyse_all(config, db_url).await?,
        Commands::AnalyseOne { config, repo } => {
            let config_str = std::fs::read_to_string(config)?;
            let config_parsed: PanslopConfig = toml::from_str(&config_str)?;
            let local_repo = GhRemoteRepo::new(repo).clone().await?;
            analyser::analyse_one(&config_parsed, &local_repo, true, None, None).await?;
        }
        Commands::Reconsider { config } => reconsider(config, db_url).await?,
        Commands::Update { config } => update::update_all(config, db_url).await?,
        Commands::HamIngress { config } => ingress::ingress_ham(config, db_url).await?,
        Commands::Version {} => println!(
            "Panslopticon v{} - Copyright (c) 2026 Mel Young. MPL 2.0.\nUpstream: https://forgejo.mlyoung.cool/mel/panslopticon",
            VERSION
        ),
    };

    info!("Process completed.");

    Ok(())
}
