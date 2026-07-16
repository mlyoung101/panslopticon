// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    GitHub,
    Reddit,
    Unknown
}

#[derive(Debug, FromRow, Clone)]
pub struct IngressItem {
    id: u64,
    url: String,
    date_added: chrono::NaiveDateTime,
    origin_platform: Platform,
    origin_src: String
}

#[derive(Debug, FromRow, Clone)]
pub struct SlopItem {
    id: u64,
    url: String,
    date_added: chrono::NaiveDateTime,
    date_last_seen: chrono::NaiveDateTime,
    dataset_path: Option<String>,
    origin_platform: Platform,
    origin_src: String
}

#[derive(Debug, FromRow, Clone)]
pub struct NotSlopItem {
    id: u64,
    url: String,
    date_added: chrono::NaiveDateTime,
}

////

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDetect {
    emojis: String,
    excessive_commits: u32,
    excessive_readme_length: u32,
    excessive_emojis: u32,
    emdash_limit: u32,
    excessive_commit_length: u32,
    readme_signals: Vec<String>,
    commit: HashMap<String, Vec<String>>,
    files: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigIngress {
    gh_tags: Vec<String>,
    subreddits: Vec<String>,
    reddit_signals: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PanslopConfig {
    detect: ConfigDetect,
    ingress: ConfigIngress,
}
