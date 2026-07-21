// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use chrono::Utc;
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
    pub id: i64,
    pub url: String,
    pub date_added: String,
    pub origin_platform: String,
    pub origin_src: String
}

#[derive(Debug, FromRow, Clone)]
pub struct SlopItem {
    pub id: i64,
    pub url: String,
    pub date_added: chrono::DateTime<Utc>,
    pub score: f64,
    pub panslop_version: String,
    pub date_last_seen: chrono::DateTime<Utc>,
    pub dataset_path: Option<String>,
    pub origin_platform: Platform,
    pub origin_src: String
}

#[derive(Debug, FromRow, Clone)]
pub struct NotSlopItem {
    pub id: u64,
    pub url: String,
    pub date_added: chrono::DateTime<Utc>,
    pub score: f64,
}

///

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDetect {
    pub emojis: String,
    pub excessive_commits_thresh: u32,
    pub excessive_readme_length: u32,
    pub excessive_commit_length: u32,
    pub readme_signals: Vec<String>,
    pub commit: HashMap<String, Vec<String>>,
    pub files: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigIngress {
    pub gh_tags: Vec<String>,
    pub subreddits: Vec<String>,
    pub gh_min_stars: u32,
    pub gh_date_cutoff: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigScoring {
    pub ai_commit: f64,
    pub readme_signal: f64,
    pub emoji: f64,
    pub emdash: f64,
    pub excessively_long_readme: f64,
    pub excessively_long_commit: f64,
    pub threshold: f64,
    pub ai_file: f64,
    pub ai_file_hidden: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PanslopConfig {
    pub detect: ConfigDetect,
    pub ingress: ConfigIngress,
    pub scoring: ConfigScoring,
}
