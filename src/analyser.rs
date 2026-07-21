// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use indicatif::ProgressIterator;
use regex::Regex;
use regex_cache::LazyRegex;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{collections::HashSet, fs, io, path::{Path, PathBuf}};
use subprocess::{Exec, Redirection};

use lazy_static::lazy_static;
use log::{debug, info, warn};

use crate::types::{IngressItem, PanslopConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref EMDASH_REGEX: Regex = Regex::new("–|—|⸺|⸻").unwrap();
    static ref GIT_COMMIT_REGEX: Regex =
        Regex::new(r#"commit ([a-z0-9]+)\nAuthor: (.*)\nDate:\s+(.*)\n\n([\S\s]*)"#).unwrap();
}

fn is_readme(path: &Path) -> bool {
    let local = path.to_str();
    if let Some(l) = local {
        l.to_lowercase().contains("readme")
    } else {
        false
    }
}

fn compile_mega_regex(regexes: &Vec<String>) -> color_eyre::Result<LazyRegex> {
    let mut mega_regex = String::new();

    for reg in regexes {
        mega_regex.push_str(&format!("({})|", reg));
    }
    mega_regex.pop();
    debug!("Mega regex: {}", mega_regex);

    Ok(LazyRegex::new(&mega_regex)?)
}

fn process_readme(config: &PanslopConfig, path: &PathBuf) -> color_eyre::Result<f64> {
    let all_paths: Vec<PathBuf> = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let maybe_readme_path = all_paths.iter().find(|it| is_readme(it));
    let signals_regex = compile_mega_regex(&config.detect.readme_signals)?;

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
        if length > config.detect.excessive_readme_length.try_into()? {
            score += config.scoring.excessively_long_readme;
        }

        score += (emojis as f64) * config.scoring.emoji;
        score += (emdashes as f64) * config.scoring.emdash;
        score += (signals as f64) * config.scoring.readme_signal;

        Ok(score)
    } else {
        warn!("Could not find README for {}", path.to_string_lossy());
        Ok(0.0)
    }
}

/// Process a single commit by checking regexes and length; returns a score update and a list of
/// detected agents
fn process_commit(
    config: &PanslopConfig,
    commit: &str,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    let mut score = 0.0;
    let mut detected_agents = HashSet::new();

    // look for each agent (try to cache regexes for a bit of a speed boost)
    for (agent, regexes) in &config.detect.commit {
        let regex = compile_mega_regex(regexes)?;

        let matches = regex.captures_iter(commit).count();
        if matches > 0 {
            detected_agents.insert(agent.to_string());
        }
        score += (matches as f64) * config.scoring.ai_commit;
    }

    if (commit.trim().len() as u32) >= config.detect.excessive_commit_length {
        score += config.scoring.excessively_long_commit;
    }

    Ok((score, detected_agents))
}

/// Process all commits in a repo. Returns the score update and a list of detected agents.
fn process_commits(
    config: &PanslopConfig,
    repo: &Path,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    // git --no-pager -C /home/mel/workspace/slop/devwebui log
    // find all commits
    let stdout = Exec::cmd("git")
            .arg("--no-pager")
            .arg("-C")
            .arg(repo.to_string_lossy().to_string())
            .arg("log")
            .arg("--pretty=oneline")
            .checked()
            .capture()?
            .stdout;
    let commits = String::from_utf8_lossy(&stdout);

    let mut score = 0.0;
    let mut detected_agents: HashSet<String> = HashSet::new();

    // only consider the last 2000 commits
    let lines: Vec<String> = commits.lines().map(|x| x.to_string()).take(2000).collect();

    info!("Now processing commits...");

    // parse each commit
    for line in lines.iter().progress() {
        let hash: String = line.split(" ").take(1).collect();

        // get the message for the commit
        // git --no-pager -C /home/mel/workspace/slop/devwebui log --format=%B -n 1 8866e9209648b45cf2dfc438ad79b1a2721ca433
        // https://stackoverflow.com/a/3357357/5007892

        let msg = String::from_utf8(
            Exec::cmd("git")
                .arg("--no-pager")
                .arg("-C")
                .arg(repo.to_string_lossy().to_string())
                .arg("log")
                .arg("--format=%B")
                .arg("-n")
                .arg("1")
                .arg(hash)
                .checked()
                .capture()?
                .stdout,
        )?;

        let (score_update, detected) = process_commit(config, &msg)?;
        score += score_update;
        detected_agents.extend(detected);
    }

    // we need to make sure that we normalise the score increment by the number of commits in the
    // repo; otherwise repos with very long history would get unexpectedly high scores
    if score > 0.0 {
        score /= lines.len() as f64;
    }

    Ok((score, detected_agents))
}

/// Removes an item from the ingress queue
async fn dequeue_item(id: i64, db: &Pool<Sqlite>) -> color_eyre::Result<()> {
    let _ = sqlx::query!("DELETE FROM ingress WHERE id = ?;", id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn analyse_one(
    config: &PanslopConfig,
    repo: &PathBuf,
    debug: bool,
    maybe_db: Option<&Pool<Sqlite>>,
    maybe_item: Option<&IngressItem>,
) -> color_eyre::Result<()> {
    let mut score = process_readme(config, repo)?;
    info!("Score after processing readme: {}", score);

    let (score_update, detected_agents) = process_commits(config, repo)?;
    score += score_update;
    info!("Score after processing all commits: {}", score);

    // don't do SQL stuff in debug mode
    if debug {
        return Ok(());
    }

    // these must exist now, since we're not in debug mode
    let db = maybe_db.unwrap();
    let item = maybe_item.unwrap();

    // in either case, dequeue the item
    dequeue_item(item.id, db).await?;

    let now = Utc::now();

    // was it slop?! the big decision!!
    if score >= config.scoring.threshold {
        info!("Slop detected!! Repo: {}", item.url);

        sqlx::query!(
            r#"
                INSERT INTO slop
                    (url, date_added, score, panslop_version, date_last_seen, dataset_path, origin_platform,
                     origin_src)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            item.url,
            now,
            score,
            VERSION,
            now,
            "",
            item.origin_platform,
            item.origin_src
        )
        .execute(db)
        .await?;

        // what was the ID we just inserted?
        // FIXME this seems stupid, can't we get it from the query above?
        let id = sqlx::query!("SELECT id FROM slop ORDER BY id DESC;")
            .fetch_one(db)
            .await?
            .id;

        for agent in detected_agents {
            sqlx::query!(
                "INSERT INTO agents(slop_id, agent) VALUES (?, ?);",
                id,
                agent
            )
            .execute(db)
            .await?;
        }
    } else {
        info!("Repo '{}' is NOT slop", item.url);
        sqlx::query!(
            r#"
                INSERT INTO not_slop (url, date_added, score) VALUES (?, ?, ?);
            "#,
            item.url,
            now,
            score
        )
        .execute(db)
        .await?;
    }

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
                    .stdout(Redirection::Null)
                    .stderr(Redirection::Null)
                    .join()?;

                info!("... Done");

                let result = analyse_one(
                    &config_parsed,
                    &tempdir.path().to_path_buf(),
                    false,
                    Some(&db),
                    Some(&row),
                )
                .await;

                match result {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("Failed to process repo '{}': {}. Removing from ingress queue.", row.url, err);
                        dequeue_item(row.id, &db).await?;
                        continue;
                    }
                }
            }
            Err(err) => {
                info!("Assuming ingress queue is done, error was: {}", err);
                break;
            }
        }
    }

    Ok(())
}
