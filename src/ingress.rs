// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashSet, path::PathBuf, time::Duration};

use chrono::{DateTime, Utc};
use log::{info, warn};
use octocrab::models::Repository;
use sqlx::{PgPool, Pool, Postgres};

use crate::{
    analyser::{calculate_score, update_full_text},
    repo::GhRemoteRepo,
    types::PanslopConfig,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checks if the given URL is in the not slop table (or the ham table)
async fn is_definitely_not_slop(url: &String, db: &Pool<Postgres>) -> color_eyre::Result<bool> {
    let not_slop_result = sqlx::query!(
        r#"SELECT COUNT(*) AS count FROM not_slop WHERE url = $1;"#,
        url
    )
    .fetch_one(db)
    .await?
    .count
    .expect("no count for not_slop");

    let ham_result = sqlx::query!(r#"SELECT COUNT(*) AS count FROM ham WHERE url = $1;"#, url)
        .fetch_one(db)
        .await?
        .count
        .expect("no count for ham");

    Ok(not_slop_result > 0 || ham_result > 0)
}

/// Checks if the given URL is already in the slop table
async fn is_already_slop(url: &String, db: &Pool<Postgres>) -> color_eyre::Result<bool> {
    let result = sqlx::query!(r#"SELECT COUNT(*) AS count FROM slop WHERE url = $1;"#, url)
        .fetch_one(db)
        .await?
        .count
        .expect("no count for slop");

    Ok(result > 0)
}

/// Ingresses one repo
async fn do_ingress_repo(
    config: &PanslopConfig,
    db: &Pool<Postgres>,
    repo: &Repository,
    source: &String,
) -> color_eyre::Result<()> {
    let now = Utc::now().naive_utc();
    let url = repo.html_url.as_ref().expect("no HTML URL");
    let creation_date = repo.created_at.expect("no creation date");
    let cutoff = DateTime::parse_from_rfc2822(&config.ingress.gh_date_cutoff)?;

    // apply some filters first
    if creation_date < cutoff {
        info!("Reject repo '{}', created before cutoff date", url);
        return Ok(());
    } else {
        info!("Accept repo: {}", url);
    }

    if is_definitely_not_slop(&url.to_string(), &db).await? {
        info!("Repo {} is in not_slop table, skipping", url);
        return Ok(());
    }

    if is_already_slop(&url.to_string(), &db).await? {
        info!("Repo {} already considered slop", url);
        return Ok(());
    }

    let insert = sqlx::query!(
        r#"
            INSERT INTO ingress(url, date_added, origin_platform, origin_src)
            VALUES ($1, $2, $3, $4);"#,
        url.as_str(),
        now,
        "github",
        source
    )
    .execute(db)
    .await;

    match insert {
        Ok(_) => {}
        Err(error) => {
            warn!("Failed to insert repo {}: {}", url, error);
        }
    }

    Ok(())
}

pub async fn ingress_gh(config: PathBuf, db_url: String) -> color_eyre::Result<()> {
    info!(
        "Start GitHub ingress process. Config: {}, DB URL: {}",
        config.to_string_lossy(),
        db_url
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    let api = octocrab::instance();
    let db = PgPool::connect(&db_url.clone()).await?;

    for topic in &config_parsed.ingress.gh_tags {
        info!("Query GitHub topic: {}", topic);
        let source = format!("tag-{}", &topic);

        let result = api
            .search()
            .repositories(&format!(
                "topic:{} stars:>={}",
                topic, config_parsed.ingress.gh_min_stars
            ))
            .per_page(100)
            .sort("updated")
            .send()
            .await?;

        for repo in &result.items {
            do_ingress_repo(&config_parsed, &db, &repo, &source).await?;
        }

        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_secs(10)); // rate limits!!
    }

    info!("Perform broad search");
    for _ in 1..config_parsed.ingress.gh_broad_search_num {
        let page = rand::random_range(1..config_parsed.ingress.gh_broad_search_pages_max);

        info!("Query GitHub newest, page: {}", page);
        let result = api
            .search()
            .repositories(&format!("stars:>={}", config_parsed.ingress.gh_min_stars))
            .per_page(100)
            .page(page as u32)
            .sort("updated")
            .send()
            .await?;

        for repo in &result.items {
            do_ingress_repo(&config_parsed, &db, &repo, &"broad_search".to_string()).await?;
        }

        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_secs(10)); // rate limits!!
    }

    Ok(())
}

/// Tries to find "good" readmes for the "ham" dataset
pub async fn ingress_ham(config: PathBuf, db_url: String) -> color_eyre::Result<()> {
    info!(
        "Start GitHub ham ingress process. Config: {}, DB URL: {}",
        config.to_string_lossy(),
        db_url
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    let api = octocrab::instance();

    let db = PgPool::connect(&db_url.clone()).await?;

    let resolved_tags: HashSet<&String> = config_parsed
        .ingress
        .gh_tags
        .difference(&config_parsed.ingress.gh_ham_tags_blocklist)
        .collect();

    info!("Ham: {:?}", resolved_tags);

    for topic in &resolved_tags {
        info!("Query GitHub topic: {}", topic);
        let page = rand::random_range(1..25);

        let result = api
            .search()
            .repositories(&format!("topic:{} stars:>90 created:<2022", topic))
            .sort("stars")
            .page(page as u32)
            .send()
            .await?;

        for repo in result.items {
            let now = Utc::now().naive_utc();
            let url = repo.html_url.expect("no HTML URL");

            if is_already_slop(&url.to_string(), &db).await? {
                warn!("Good repo {} is already somehow slop? Skipping.", url);
                continue;
            }

            if is_definitely_not_slop(&url.to_string(), &db).await? {
                warn!("Repo {} already recorded, skipping", url);
                continue;
            }

            info!("Confirming repo {} is not slop", url);
            let remote = GhRemoteRepo::new(url.to_string());
            let local = remote.clone().await?;

            let Ok((score, _)) = calculate_score(&config_parsed, &local).await else {
                warn!("Failed to calculate score for ham repo: {}", url);
                continue;
            };

            if score <= config_parsed.scoring.ham_threshold {
                info!("Repo {} is confirmed ham, processing full text", url);

                sqlx::query!(
                r#"
                    INSERT INTO ham (url, date_added, score, panslop_version, origin_platform, origin_src)
                    VALUES ($1, $2, $3, $4, $5, $6);
                "#,
                    url.to_string(),
                    now,
                    score as f32,
                    VERSION,
                    "github",
                    format!("tag-{}", topic)
                )
                .execute(&db)
                .await?;

                // what was the ID we just inserted?
                let id = sqlx::query!("SELECT id FROM ham ORDER BY id DESC;")
                    .fetch_one(&db)
                    .await?
                    .id;

                let Ok(_) = update_full_text(id, &local, &db, true).await else {
                    warn!("Failed to update full text for ham repo: {}", url);
                    continue;
                };
            } else {
                warn!(
                    "Ham repo {} is in fact slop!! (score={}), skipping",
                    url, score
                );
                continue;
            }
        }

        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_secs(10)); // rate limits!!
    }

    Ok(())
}
