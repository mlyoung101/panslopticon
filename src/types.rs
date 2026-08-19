// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    GitHub,
    Reddit,
    Unknown,
}

#[derive(Debug, FromRow, Clone)]
pub struct IngressItem {
    pub id: i64,
    pub url: String,
    pub date_added: String,
    pub origin_platform: String,
    pub origin_src: String,
}

#[derive(Debug, FromRow, Clone, Type)]
pub struct SlopItem {
    pub id: i64,
    pub url: String,
    pub date_added: NaiveDateTime,
    pub score: f64,
    pub panslop_version: String,
    pub date_last_seen: NaiveDateTime,
    pub dataset_path: Option<String>,
    pub origin_platform: String,
    pub origin_src: String,
    #[sqlx(try_from = "i64")]
    pub dead: bool,
}

// alright, bullshit #2 in the hell world that is sqlx
// when we made the ham table, we forgot to make all the fields 'NOT NULL'
// and unfortunately SQLite doesn't let us make a NULL column NOT NULL without completely redoing it
// HENCE, everything here, literally everything, has to be Option.
// I extend my gratitudes to SQLx for producing a proc macro error so fucking incomprehensible it
// only took half an hour to figure out! amazing!
#[derive(Debug, FromRow, Clone, Type)]
pub struct HamItem {
    pub id: i64,
    pub url: Option<String>,
    pub date_added: Option<String>,
    pub score: f64,
    pub panslop_version: Option<String>,
    pub origin_platform: Option<String>,
    pub origin_src: Option<String>,
    #[sqlx(try_from = "i64")]
    pub dead: bool,
    pub date_last_seen: Option<String>,
}

#[derive(Debug, FromRow, Clone)]
pub struct FullTextItem {
    pub slop_id: i64,
    pub file: String,
    pub text: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct HamFullTextItem {
    pub id: i64,
    pub file: String,
    pub text: String,
}

///

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigDetect {
    pub emojis: String,
    pub excessive_readme_length: u32,
    pub excessive_commit_length: u32,
    pub readme_signals: Vec<String>,
    pub commit: HashMap<String, Vec<String>>,
    pub files: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigIngress {
    pub gh_tags: HashSet<String>,
    pub gh_ham_tags_blocklist: HashSet<String>,
    pub gh_min_stars: u32,
    pub gh_date_cutoff: String,
    pub gh_http_head_wait_ms: u64,
    pub gh_broad_search_num: i32,
    pub gh_broad_search_pages_max: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigScoring {
    pub ai_commit: f64,
    pub readme_signal: f64,
    pub emoji: f64,
    pub emdash: f64,
    pub excessively_long_readme: f64,
    pub excessively_long_commit: f64,
    pub threshold: f64,
    pub ai_file: f64,
    pub ai_file_hidden: f64,
    pub readme_multiplier: f64,
    pub ham_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PanslopConfig {
    pub detect: ConfigDetect,
    pub ingress: ConfigIngress,
    pub scoring: ConfigScoring,
}
