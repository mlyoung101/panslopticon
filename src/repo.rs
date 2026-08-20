// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs, io, path::PathBuf};

use git2::{Repository, Sort};
use log::{debug, info};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use subprocess::{Exec, Redirection};
use tempfile::TempDir;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A remote GitHub repo
pub struct GhRemoteRepo {
    pub url: String,
}

/// A local cloned GitHub repo
pub struct GhLocalRepo {
    pub path: TempDir,
}

impl GhRemoteRepo {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn clone(&self) -> color_eyre::Result<GhLocalRepo> {
        let tempdir = tempfile::Builder::new()
            .prefix("panslop_ingress_")
            .tempdir()?;

        info!("Cloning...");
        // based on:
        // https://codeberg.org/polyphony/repo-slopscore/src/branch/main/src/git/clone.rs#L39
        Exec::cmd("git")
            .arg("clone")
            .arg("--sparse")
            .arg("--single-branch")
            .arg("--filter=tree:0")
            .arg(format!("{}.git", self.url))
            .arg(tempdir.path().to_string_lossy().to_string())
            .checked()
            .stdout(Redirection::Null)
            .stderr(Redirection::Null)
            .join()?;

        Ok(GhLocalRepo { path: tempdir })
    }

    pub async fn exists(&self) -> color_eyre::Result<bool> {
        let user_agent = format!(
            "Mozilla/5.0 (compatible; Panslopticon-Update/{}; +https://forgejo.mlyoung.cool/mel/panslopticon)",
            VERSION
        );
        let policy = ExponentialBackoff::builder().build_with_max_retries(10);

        // original reqwest client
        let reqwest_client = Client::builder().user_agent(user_agent).build()?;
        // middleware client
        let client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(policy))
            .build();

        // let client = reqwest::Client::builder().user_agent(user_agent).build()?;
        let status = client.head(&self.url).send().await?;
        Ok(status.status().is_success())
    }

    // TODO get stars count
}

impl GhLocalRepo {
    // we can't do this because we use a temp path :/
    // pub fn new(path: PathBuf) -> Self {
    //     Self { path }
    // }

    pub fn get_commit_messages(&self, max: u32) -> color_eyre::Result<Vec<String>> {
        let path = format!("{}/.git", self.path.path().to_string_lossy());
        debug!("Get commits for: {}", path);
        let repo = Repository::open(&path)?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME )?;
        // this one is EXTREMELY important!! otherwise the revwalk does NOTHING!!
        revwalk.push_head()?;

        let mut out: Vec<String> = Vec::new();
        let mut i = 0;

        for c in revwalk {
            let commit = repo.find_commit(c?)?;
            let author = commit.author().to_string();
            let msg = commit.message()?;
            out.push(format!("{}\n{}", author, msg));
            if i > max {
                break;
            }
            i += 1;
        }

        Ok(out)
    }

    pub fn get_all_paths(&self) -> color_eyre::Result<Vec<PathBuf>> {
        let all_paths: Vec<PathBuf> = fs::read_dir(&self.path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        Ok(all_paths)
    }
}
