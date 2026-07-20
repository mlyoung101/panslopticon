// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};
use tokio;

use crate::types::PanslopConfig;

pub mod analyser;
pub mod ingress;
pub mod types;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Subcommand)]
enum Commands {
    /// Ingress from GitHub
    #[command()]
    GHIngress {
        /// Config TOML path
        config: PathBuf,

        /// Database path
        db: PathBuf,
    },

    /// Ingress from Reddit
    #[command()]
    RedditIngress {
        /// Config TOML path
        config: PathBuf,

        /// Database path
        db: PathBuf,
    },

    /// Analyse all previously ingressed data
    #[command()]
    Analyse {
        /// Config TOML path
        config: PathBuf,

        /// Database path
        db: PathBuf,
    },

    /// Analyse a single repository
    #[command()]
    AnalyseOne {
        /// Config TOML path
        config: PathBuf,

        /// Repo path
        repo: PathBuf,
    },

    /// Update statistics about existing repositories
    #[command()]
    UpdateStats {
        /// Config TOML path
        config: PathBuf,

        /// Database path
        db: PathBuf,
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

    match args.command {
        Commands::GHIngress { config, db } => ingress::ingress_gh(config, db).await?,
        Commands::RedditIngress { config, db } => ingress::ingress_reddit(config, db).await?,
        Commands::Analyse { config, db } => _ = analyser::analyse_all(config, db).await?,
        Commands::AnalyseOne { config, repo } => {
            let config_str = std::fs::read_to_string(config)?;
            let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

            _ = analyser::analyse_one(&config_parsed, &repo, true).await?;
        }
        Commands::UpdateStats { config, db } => todo!(),
        Commands::Version {} => println!(
            "Panslopticon v{} - Copyright (c) 2026 Mel Young. MPL 2.0.\nUpstream: https://codeberg.org/melyoung/panslopticon",
            VERSION
        ),
    };

    Ok(())
}
