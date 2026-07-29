// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, time::Duration};

use chrono::Utc;
use log::{info, warn};
use sqlx::SqlitePool;

use crate::{
    analyser::update_full_text,
    repo::GhRemoteRepo,
    types::{PanslopConfig, SlopItem},
};

pub async fn update_all(config: PathBuf, db: PathBuf) -> color_eyre::Result<()> {
    info!(
        "Start update of all existing items. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    let url = format!("sqlite://{}", db.to_string_lossy());
    let db = SqlitePool::connect(&url).await?;

    // id INTEGER PRIMARY KEY NOT NULL,
    // url TEXT NOT NULL,
    // date_added TEXT NOT NULL,
    // score REAL NOT NULL, -- why this was detected, the score
    // panslop_version TEXT NOT NULL, -- version of panslopticon that detected this
    // date_last_seen TEXT NOT NULL,
    // dataset_path TEXT, -- Zstd compressed storage location on disk, once checked out
    // origin_platform TEXT NOT NULL, -- i.e. github, reddit
    // origin_src TEXT NOT NULL -- i.e. r/selfhosted; tag-llm

    // due to bullshit
    // https://github.com/transact-rs/sqlx/issues/2648#issuecomment-1970636631
    let slop = sqlx::query_as!(
        SlopItem,
        r#"
            SELECT
        id, url, date_added, score, panslop_version, date_last_seen, dataset_path, origin_platform, origin_src,
        dead AS "dead: _"
            FROM slop
            ORDER BY RANDOM();
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
            let now = Utc::now();
            sqlx::query!(
                "UPDATE slop SET date_last_seen = ? WHERE id = ?;",
                now,
                item.id
            )
            .execute(&db)
            .await?;
        } else {
            warn!("No LONGER exists! Marking as dead.");
            sqlx::query!("UPDATE slop SET dead = 1 WHERE id = ?", item.id)
                .execute(&db)
                .await?;
        }

        // wait for HIDDEN(!) rate limits
        info!("Waiting for rate limit...");
        std::thread::sleep(Duration::from_millis(
            config_parsed.ingress.gh_http_head_wait_ms,
        ));
    }

    Ok(())
}
