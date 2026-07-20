// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use color_eyre::eyre::eyre;
use regex::Regex;
use regex_cache::LazyRegex;
use sqlx::SqlitePool;
use std::{collections::HashSet, fs, io, path::PathBuf};
use subprocess::Exec;
use tempfile::NamedTempFile;

use lazy_static::lazy_static;
use log::{debug, info};

use crate::types::{IngressItem, PanslopConfig};

lazy_static! {
    static ref EMDASH_REGEX: Regex = Regex::new("–|—|⸺|⸻").unwrap();
    static ref GIT_COMMIT_REGEX: Regex =
        Regex::new(r#"commit ([a-z0-9]+)\nAuthor: (.*)\nDate:\s+(.*)\n\n([\S\s]*)"#).unwrap();
}

fn is_readme(path: &PathBuf) -> bool {
    let local = path.to_str();
    return if let Some(l) = local {
        l.to_lowercase().contains("readme")
    } else {
        false
    };
}

fn compile_readme_signal_mega_regex(config: &PanslopConfig) -> color_eyre::Result<LazyRegex> {
    let mut mega_regex = String::new();

    for reg in &config.detect.readme_signals {
        mega_regex.push_str(&format!("({})|", &reg));
    }
    mega_regex.pop();
    debug!("Mega regex: {}", mega_regex);

    Ok(LazyRegex::new(&mega_regex)?)
}

fn process_readme(config: &PanslopConfig, path: &PathBuf) -> color_eyre::Result<f64> {
    let all_paths: Vec<PathBuf> = fs::read_dir(&path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let maybe_readme_path = all_paths.iter().find(|it| is_readme(it));
    let signals_regex = compile_readme_signal_mega_regex(&config)?;

    if let Some(readme_path) = &maybe_readme_path {
        info!("README: {}", readme_path.to_string_lossy());

        let readme = fs::read_to_string(readme_path)?;
        let emdashes = EMDASH_REGEX.captures_iter(&readme).count();
        let emojis = Regex::new(&config.detect.emojis)?
            .captures_iter(&readme)
            .count();
        let length = readme.len();
        let signals = signals_regex.captures_iter(&readme).count();

        info!("Emdashes: {}", emdashes);
        info!("Emojis: {}", emojis);
        info!("Readme length: {}", length);
        info!("Readme signals (regex): {}", signals);

        signals_regex
            .captures_iter(&readme)
            .for_each(|x| info!("    Readme signal: '{}'", x.get_match().as_str()));

        // calculate score increment
        let mut score = 0.0;
        if length > config.detect.excessive_readme_length.try_into().unwrap() {
            score += config.scoring.excessively_long_readme;
        }

        score += (emojis as f64) * config.scoring.emoji;
        score += (emdashes as f64) * config.scoring.emdash;
        score += (signals as f64) * config.scoring.readme_signal;

        Ok(score)
    } else {
        return Err(eyre!(
            "Could not find README for {}",
            path.to_string_lossy()
        ));
    }
}

/// Process a single commit by checking regexes and length; returns a score update and a list of
/// detected agents
fn process_commit(
    config: &PanslopConfig,
    commit: &String,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    let mut score = 0.0;
    let mut detected_agents = HashSet::new();
    for (agent, regexes) in &config.detect.commit {
        for regex in regexes {
            let matches = LazyRegex::new(regex)?.captures_iter(commit).count();
            score += (matches as f64) * config.scoring.ai_commit;
            detected_agents.insert(agent.to_string());
        }
    }

    Ok((score, detected_agents))
}

fn process_commits(config: &PanslopConfig, repo: &PathBuf) -> color_eyre::Result<f64> {
    // git --no-pager -C /home/mel/workspace/slop/devwebui log
    // find all commits
    let commits = String::from_utf8(
        Exec::cmd("git")
            .arg("--no-pager")
            .arg("-C")
            .arg(&repo.to_string_lossy().to_string())
            .arg("log")
            .arg("--pretty=oneline")
            .checked()
            .capture()?
            .stdout,
    )?;

    let mut score = 0.0;

    // parse each commit
    for line in commits.lines() {
        let hash: String = line.split(" ").take(1).collect();

        // get the message for the commit
        // git --no-pager -C /home/mel/workspace/slop/devwebui log --format=%B -n 1 8866e9209648b45cf2dfc438ad79b1a2721ca433
        // https://stackoverflow.com/a/3357357/5007892

        let msg = String::from_utf8(
            Exec::cmd("git")
                .arg("--no-pager")
                .arg("-C")
                .arg(&repo.to_string_lossy().to_string())
                .arg("log")
                .arg("--format=%B")
                .arg("-n")
                .arg("1")
                .arg(hash)
                .checked()
                .capture()?
                .stdout,
        )?;

        let (score_update, detected_agents) = process_commit(&config, &msg)?;
        score += score_update;

        // debug!("Commit {}: {}", hash, msg);
    }

    Ok(score)
}

pub async fn analyse_one(
    config: &PanslopConfig,
    repo: &PathBuf,
    debug: bool,
) -> color_eyre::Result<()> {
    let mut score = process_readme(config, repo)?;
    info!("Score after processing readme: {}", score);

    score += process_commits(config, repo)?;
    info!("Score after processing all commits: {}", score);

    Ok(())
}

pub async fn analyse_all(config: PathBuf, db: PathBuf) -> color_eyre::Result<()> {
    info!(
        "Start analysis of all ingress items. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    let url = format!("sqlite://{}", db.to_string_lossy());
    let db = SqlitePool::connect(&url).await?;

    loop {
        let maybe_row = sqlx::query_as!(
            IngressItem,
            r#"
        SELECT id, `url`, date_added, origin_platform, origin_src FROM ingress LIMIT 1;
            "#
        )
        .fetch_one(&db)
        .await;

        match maybe_row {
            Ok(row) => {
                let tempdir = tempfile::Builder::new()
                    .prefix("panslop_ingress_")
                    .tempdir()?;

                info!("Checkout: {}", row.url);

                // based on:
                // https://codeberg.org/polyphony/repo-slopscore/src/branch/main/src/git/clone.rs#L39
                Exec::cmd("git")
                    .arg("clone")
                    .arg("--sparse")
                    .arg("--single-branch")
                    .arg("--filter=tree:0")
                    .arg(format!("{}.git", row.url))
                    .arg(tempdir.path().to_string_lossy().to_string())
                    .checked()
                    .join()?;

                info!("... Done");

                return Ok(
                    analyse_one(&config_parsed, &tempdir.path().to_path_buf(), false).await?,
                );
            }
            Err(err) => {
                info!("Assuming ingress queue is done, error was: {}", err);
                break;
            }
        }
    }

    Ok(())
}
