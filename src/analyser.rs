// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::Utc;
use dialoguer::Confirm;
use indicatif::ProgressIterator;
use regex::Regex;
use regex_cache::LazyRegex;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use log::{debug, info, warn};

use crate::{
    repo::{GhLocalRepo, GhRemoteRepo},
    types::{IngressItem, PanslopConfig, SlopItem},
};

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
    // the last character is a '|' which we don't want
    mega_regex.pop();
    debug!("Mega regex: {}", mega_regex);

    Ok(LazyRegex::new(&mega_regex)?)
}

fn process_readme(config: &PanslopConfig, repo: &GhLocalRepo) -> color_eyre::Result<f64> {
    let all_paths = repo.get_all_paths()?;

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

        // normalise to readme length
        score /= readme.len() as f64;
        score *= config.scoring.readme_multiplier;

        Ok(score)
    } else {
        warn!(
            "Could not find README for {}",
            repo.path.path().to_string_lossy()
        );
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
    repo: &GhLocalRepo,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    let mut score = 0.0;
    let mut detected_agents: HashSet<String> = HashSet::new();

    info!("Now processing commits...");
    let commits = repo.get_commit_hashes(2000)?;

    // parse each commit
    for hash in commits.iter().progress() {
        let msg = repo.get_commit_message(hash)?;

        let (score_update, detected) = process_commit(config, &msg)?;
        score += score_update;
        detected_agents.extend(detected);
    }

    // we need to make sure that we normalise the score increment by the number of commits in the
    // repo; otherwise repos with very long history would get unexpectedly high scores
    if score > 0.0 {
        score /= commits.len() as f64;
    }

    Ok((score, detected_agents))
}

fn process_files(
    config: &PanslopConfig,
    repo: &GhLocalRepo,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    let all_paths = repo.get_all_paths()?;

    let mut score = 0.0;
    let mut detected_agents: HashSet<String> = HashSet::new();

    // process visible files
    for (agent, regexes) in &config.detect.files {
        let regex = compile_mega_regex(regexes)?;
        if all_paths
            .iter()
            .any(|x| regex.is_match(&x.to_string_lossy()))
        {
            // agent detected!
            info!(
                "Detected agent {} in files by query {} in visible file",
                agent, regex
            );
            score += config.scoring.ai_file;
            detected_agents.insert(agent.to_string());
        }

        // TODO process .gitignore and .dockerignore, with a separate function
    }

    Ok((score, detected_agents))
}

pub async fn update_full_text(
    id: i64,
    repo: &GhLocalRepo,
    db: &Pool<Sqlite>,
) -> color_eyre::Result<()> {
    let all_paths = repo.get_all_paths()?;

    for path in &all_paths {
        let path_str = path.file_name().unwrap().to_string_lossy().to_lowercase();

        // ignore licence files, and ensure we have a markdown file
        if path.extension().unwrap_or(OsStr::new("invalid")) != "md"
            || path_str.contains("license")
            || path_str.contains("licence")
        {
            continue;
        }

        // otherwise, we can add to the database
        let actual_path = path.file_name().unwrap().to_str().unwrap();
        let Ok(contents) = fs::read_to_string(path) else {
            warn!("Failed to read file: {}", path_str);
            continue;
        };

        sqlx::query!(
            "INSERT INTO full_text(slop_id, file, text) VALUES (?, ?, ?);",
            id,
            actual_path,
            contents
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

/// Removes an item from the ingress queue
async fn dequeue_item(id: i64, db: &Pool<Sqlite>) -> color_eyre::Result<()> {
    let _ = sqlx::query!("DELETE FROM ingress WHERE id = ?;", id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn calculate_score(
    config: &PanslopConfig,
    repo: &GhLocalRepo,
) -> color_eyre::Result<(f64, HashSet<String>)> {
    let mut score = process_readme(config, repo)?;
    info!("Score after processing readme: {}", score);

    let (commits_score, mut detected_agents) = process_commits(config, repo)?;
    score += commits_score;
    info!("Score after processing all commits: {}", score);

    let (files_score, files_agents) = process_files(config, repo)?;
    score += files_score;
    detected_agents.extend(files_agents);
    info!("Score after processing all files: {}", score);

    Ok((score, detected_agents))
}

pub async fn analyse_one(
    config: &PanslopConfig,
    repo: &GhLocalRepo,
    debug: bool,
    maybe_db: Option<&Pool<Sqlite>>,
    maybe_item: Option<&IngressItem>,
) -> color_eyre::Result<()> {
    let (score, detected_agents) = calculate_score(config, repo).await?;

    // don't do SQL stuff in debug mode
    if debug {
        return Ok(());
    }

    // these must exist now, since we're not in debug mode
    let db = maybe_db.unwrap();
    let item = maybe_item.unwrap();

    // in either case, dequeue the item
    match dequeue_item(item.id, db).await {
        Ok(_) => {}
        Err(err) => {
            warn!(
                "Failed to dequeue ingress item ID: {}. Why: {}. This is presumably because we are being called from update.rs?",
                item.id, err
            );
        }
    }

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

        update_full_text(id, repo, db).await?;
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
                info!("Try checkout: {}", row.url);
                let remote_repo = GhRemoteRepo::new(row.url.clone());
                if !remote_repo.exists().await? {
                    warn!("Repo {} no longer exists", remote_repo.url);
                    dequeue_item(row.id, &db).await?;
                    continue;
                }

                let local_repo = remote_repo.clone().await?;

                let result =
                    analyse_one(&config_parsed, &local_repo, false, Some(&db), Some(&row)).await;

                match result {
                    Ok(_) => {}
                    Err(err) => {
                        warn!(
                            "Failed to process repo '{}': {}. Removing from ingress queue.",
                            row.url, err
                        );
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

pub async fn cleanup_all(config: PathBuf, db: PathBuf) -> color_eyre::Result<()> {
    info!(
        "Start cleanup of all slop items to match new scoring. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    let url = format!("sqlite://{}", db.to_string_lossy());
    let db = SqlitePool::connect(&url).await?;

    let slop = sqlx::query_as!(
        SlopItem,
        r#"
            SELECT
        id, url, date_added, score, panslop_version, date_last_seen, dataset_path, origin_platform, origin_src
            FROM slop
            WHERE panslop_version != "0.6.0"
            ORDER BY RANDOM();
        "#
    )
    .fetch_all(&db)
    .await?;

    for item in &slop {
        info!("Try checkout: {}", item.url);
        let remote_repo = GhRemoteRepo::new(item.url.clone());
        if !remote_repo.exists().await? {
            warn!("Repo {} no longer exists", remote_repo.url);
            // don't remove, and don't clone, keep the old score
            continue;
        }

        let local_repo = remote_repo.clone().await?;
        let (new_score, _) = calculate_score(&config_parsed, &local_repo).await?;

        // edge case lol
        if new_score <= 0.01 {
            continue;
        }

        sqlx::query!(
            "UPDATE slop SET score = ? WHERE id = ?;",
            new_score,
            item.id
        )
        .execute(&db)
        .await?;

        sqlx::query!(
            "UPDATE slop SET panslop_version = ? WHERE id = ?;",
            VERSION,
            item.id
        )
        .execute(&db)
        .await?;
    }

    if Confirm::new()
        .with_prompt("Alright, shall we nuke the old data?")
        .interact()?
    {
        info!("Your funeral...");

        let rows_affected = sqlx::query!(
            "DELETE FROM slop WHERE score < ?;",
            config_parsed.scoring.threshold
        )
        .execute(&db)
        .await?
        .rows_affected();

        warn!(
            "Deleted {} slop items (original count was {})",
            rows_affected,
            &slop.len()
        );

        info!("Cleanup deleted full_text");
        let full_text_removed =
            sqlx::query!("delete from full_text where slop_id not in (select id from slop);")
                .execute(&db)
                .await?
                .rows_affected();
        info!("Removed {} full text items", full_text_removed);
    } else {
        info!("Okay, we won't do that.");
    }

    Ok(())
}
