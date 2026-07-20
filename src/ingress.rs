// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, Utc};
use log::{info, warn};
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::types::PanslopConfig;

pub async fn ingress_reddit(config: PathBuf, db: PathBuf) -> color_eyre::Result<()> {
    info!(
        "Start Reddit ingress process. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let _config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    // for sub_name in config_parsed.ingress.subreddits {
    //     info!("Checking: {}", sub_name);
    //
    //     let sub = Subreddit::new(&sub_name);
    //
    //     let new = sub.latest(10, None).await?;
    //
    //     println!("{:?}", new);
    //     break;
    // }

    Ok(())
}

/// Checks if the given URL is in the not slop table
async fn is_definitely_not_slop(url: &String, db: &Pool<Sqlite>) -> color_eyre::Result<bool> {
    let result = sqlx::query!(
        r#"SELECT COUNT(*) AS count FROM not_slop WHERE url = ?;"#,
        url
    )
    .fetch_one(db)
    .await?;

    Ok(result.count > 0)
}

/// Checks if the given URL is already in the slop table
async fn is_already_slop(url: &String, db: &Pool<Sqlite>) -> color_eyre::Result<bool> {
    let result = sqlx::query!(r#"SELECT COUNT(*) AS count FROM slop WHERE url = ?;"#, url)
        .fetch_one(db)
        .await?;

    Ok(result.count > 0)
}

pub async fn ingress_gh(config: PathBuf, db: PathBuf) -> color_eyre::Result<()> {
    info!(
        "Start GitHub ingress process. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    // TODO auth, for nicer rate limits
    let api = octocrab::instance();

    let url = format!("sqlite://{}", db.to_string_lossy());
    let db = SqlitePool::connect(&url).await?;

    let cutoff = DateTime::parse_from_rfc2822(&config_parsed.ingress.gh_date_cutoff)?;

    for topic in config_parsed.ingress.gh_tags {
        info!("Query GitHub topic: {}", topic);

        let result = api
            .search()
            .repositories(&format!(
                "topic:{} stars:>={}",
                topic, config_parsed.ingress.gh_min_stars
            ))
            .sort("updated")
            .send()
            .await?;

        for repo in result.items {
            let now = Utc::now();
            let url = repo.html_url.expect("no HTML URL");
            let creation_date = repo.created_at.expect("no creation date");

            // apply some filters first
            if creation_date < cutoff {
                info!("Reject repo '{}', created before cutoff date", url);
                continue;
            } else {
                info!("Accept repo: {}", url);
            }

            if is_definitely_not_slop(&url.to_string(), &db).await? {
                info!("Repo {} is in not_slop table, skipping", url);
                continue;
            }

            if is_already_slop(&url.to_string(), &db).await? {
                info!("Repo {} already considered slop", url);
                continue;
            }

            let insert = sqlx::query!(
                r#"
                INSERT INTO ingress(url, date_added, origin_platform, origin_src)
                VALUES (?, ?, ?, ?);"#,
                &url.as_str(),
                &now,
                "github",
                format!("tag-{}", topic)
            )
            .execute(&db)
            .await;

            match insert {
                Ok(_) => {}
                Err(error) => {
                    warn!("Failed to insert repo {}: {}", url, error);
                }
            }
        }

        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_secs(10)); // rate limits!!
    }

    Ok(())
}
