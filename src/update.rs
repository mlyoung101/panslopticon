// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, time::Duration};

use chrono::Utc;
use log::{info, warn};
use sqlx::PgPool;

use crate::{
    VERSION,
    analyser::{calculate_score, update_full_text},
    repo::GhRemoteRepo,
    types::{HamItem, NotSlopItem, PanslopConfig, SlopItem},
};

pub async fn update_all(config_path: PathBuf, db_url: String) -> color_eyre::Result<()> {
    info!(
        "Start update of all existing items. Config: {}, DB URL: {}",
        config_path.to_string_lossy(),
        db_url
    );

    let config_str = std::fs::read_to_string(config_path)?;
    let config: PanslopConfig = toml::from_str(&config_str)?;

    let db = PgPool::connect(&db_url.clone()).await?;

    // FIXME EXTREMELY UGLY CODE DUPLICATION RAHHHH

    // also, we now only fetch up to 4k repos a day (2k ham + 2k spam) to hopefully stay on GH's
    // good side and not get my IP address perma blocked

    info!("Updating spam...");
    {
        // due to bullshit
        // https://github.com/transact-rs/sqlx/issues/2648#issuecomment-1970636631
        let slop = sqlx::query_as!(
        SlopItem,
        r#"
            SELECT
        id, url, date_added, score, panslop_version, date_last_seen, dataset_path, origin_platform, origin_src,
        dead AS "dead: _"
            FROM slop
            ORDER BY RANDOM()
            LIMIT 2000;
        "#
    )
    .fetch_all(&db)
    .await?;

        for item in slop {
            info!("Update repo: {}", item.url);

            if item.dead {
                info!("Repo is already known to be dead, skipping...");
                continue;
            }

            let repo = GhRemoteRepo::new(item.url.clone());

            if repo.exists().await? {
                info!("Still exists");
                let now = Utc::now().naive_utc();
                sqlx::query!(
                    "UPDATE slop SET date_last_seen = $1 WHERE id = $2;",
                    now,
                    item.id
                )
                .execute(&db)
                .await?;

                if config.storage.download_repos {
                    // lazy, dir must already exist
                    let path =
                        PathBuf::from(format!("{}/slop/{}", config.storage.dataset_path, item.id));
                    if !path.exists() {
                        repo.clone_to(path).await?;
                    } else {
                        info!("Repo already saved");
                    }
                }
            } else {
                warn!("No LONGER exists! Marking as dead.");
                sqlx::query!("UPDATE slop SET dead = TRUE WHERE id = $1", item.id)
                    .execute(&db)
                    .await?;

                // we would be too fast, wait for rate limit
                info!("Waiting for rate limit...");
                std::thread::sleep(Duration::from_millis(config.ingress.gh_http_head_wait_ms));
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////

    info!("Updating ham...");
    {
        // due to bullshit
        // https://github.com/transact-rs/sqlx/issues/2648#issuecomment-1970636631
        let slop = sqlx::query_as!(
        HamItem,
        r#"
            SELECT
        id, url, date_added, score, panslop_version, origin_platform, origin_src, dead, date_last_seen
            FROM ham
            ORDER BY RANDOM()
            LIMIT 2000;
        "#
        )
        .fetch_all(&db)
        .await?;

        for item in &slop {
            info!("Update repo: {}", item.url.clone().unwrap());

            if item.dead {
                info!("Repo is already known to be dead, skipping...");
                continue;
            }

            let repo = GhRemoteRepo::new(item.url.clone().unwrap());

            if repo.exists().await? {
                info!("Still exists");
                let now = Utc::now().naive_utc();
                sqlx::query!(
                    "UPDATE ham SET date_last_seen = $1 WHERE id = $2;",
                    now,
                    item.id
                )
                .execute(&db)
                .await?;

                // lazy, dir must already exist
                let path =
                    PathBuf::from(format!("{}/ham/{}", config.storage.dataset_path, item.id));
                if !path.exists() {
                    repo.clone_to(path).await?;
                } else {
                    info!("Repo already saved");
                }
            } else {
                warn!("No LONGER exists! Marking as dead.");
                sqlx::query!("UPDATE ham SET dead = TRUE WHERE id = $1", item.id)
                    .execute(&db)
                    .await?;

                // we would be too fast, wait for rate limit
                info!("Waiting for rate limit...");
                std::thread::sleep(Duration::from_millis(config.ingress.gh_http_head_wait_ms));
            }
        }
    }

    Ok(())
}

pub async fn reconsider(config_path: PathBuf, db_url: String) -> color_eyre::Result<()> {
    info!(
        "Reconsider not_slop items. Config: {}, DB URL: {}",
        config_path.to_string_lossy(),
        db_url
    );

    let config_str = std::fs::read_to_string(config_path)?;
    let config: PanslopConfig = toml::from_str(&config_str)?;

    let db = PgPool::connect(&db_url.clone()).await?;

    let not_slop = sqlx::query_as!(
        NotSlopItem,
        r#"
            SELECT id, url, date_added, score
            FROM not_slop
            ORDER BY score DESC
            LIMIT 2000;
        "#
    )
    .fetch_all(&db)
    .await?;

    for item in &not_slop {
        info!("Reconsider repo: {}, score: {}", &item.url, item.score);

        let repo = GhRemoteRepo::new(item.url.clone());

        if !repo.exists().await? {
            warn!("Repo no longer exists, skipping");
            continue;
        }

        let local_repo = &repo.clone().await?;
        let (score, detected_agents) = calculate_score(&config, local_repo).await?;

        // FIXME BAD CODE DUPLICATION from analyser.rs
        let now = Utc::now().naive_utc();

        // was it slop?! the big decision!!
        if score >= config.scoring.threshold {
            info!("Slop detected!! Repo: {}", item.url);

            // remove from not_slop
            sqlx::query!("DELETE FROM not_slop WHERE id = $1;", item.id).execute(&db).await?;

            let id = sqlx::query!(
                r#"
                    INSERT INTO slop
                        (url, date_added, score, panslop_version, date_last_seen, dataset_path, origin_platform,
                         origin_src, dead)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)
                    RETURNING id;
                "#,
                item.url,
                now,
                score as f32,
                VERSION,
                now,
                "",
                "github",
                "reconsider_not_slop"
            )
            .fetch_one(&db)
            .await?.id;

            for agent in detected_agents {
                sqlx::query!(
                    "INSERT INTO agents(slop_id, agent) VALUES ($1, $2);",
                    id,
                    agent
                )
                .execute(&db)
                .await?;
            }

            update_full_text(id, local_repo, &db, false).await?;
        } else {
            info!("Repo '{}' is STILL NOT slop", &item.url.clone());
        }

        // wait for HIDDEN(!) rate limits
        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_millis(config.ingress.gh_http_head_wait_ms));
    }
    Ok(())
}
