// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, time::Duration};

use chrono::Utc;
use log::{info, warn};
use sqlx::{PgPool, Postgres};

use crate::{
    repo::GhRemoteRepo,
    types::{HamItem, PanslopConfig, SlopItem},
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
    let db = PgPool::connect(&url).await?;

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
                let now = Utc::now();
                sqlx::query!(
                    "UPDATE slop SET date_last_seen = $1 WHERE id = $2;",
                    // FIXME
                    now,
                    item.id
                )
                .execute(&db)
                .await?;
            } else {
                warn!("No LONGER exists! Marking as dead.");
                sqlx::query!("UPDATE slop SET dead = 1 WHERE id = $1", item.id)
                    .execute(&db)
                    .await?;
            }

            // wait for HIDDEN(!) rate limits
            info!("Waiting for rate limit...");
            std::thread::sleep(Duration::from_millis(
                config_parsed.ingress.gh_http_head_wait_ms,
            ));
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
        id, url, date_added, score, panslop_version, origin_platform, origin_src, dead AS "dead: _", date_last_seen
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
            } else {
                warn!("No LONGER exists! Marking as dead.");
                sqlx::query!("UPDATE ham SET dead = 1 WHERE id = $1", item.id)
                    .execute(&db)
                    .await?;
            }

            // wait for HIDDEN(!) rate limits
            info!("Waiting for rate limit...");
            std::thread::sleep(Duration::from_millis(
                config_parsed.ingress.gh_http_head_wait_ms,
            ));
        }
    }

    Ok(())
}
