// Copyright (c) 2026 Mel Young.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
// was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use color_eyre::eyre::eyre;
use regex::Regex;
use std::{fs, io, path::PathBuf};

use lazy_static::lazy_static;
use log::{debug, info};

use crate::types::PanslopConfig;

lazy_static! {
    static ref EMDASH_REGEX: Regex = Regex::new("–|—|⸺|⸻").unwrap();
}

fn is_readme(path: &PathBuf) -> bool {
    let local = path.to_str();
    return if let Some(l) = local {
        l.to_lowercase().contains("readme")
    } else {
        false
    };
}

fn compile_readme_signal_mega_regex(config: &PanslopConfig) -> color_eyre::Result<Regex> {
    let mut mega_regex = String::new();

    for reg in &config.detect.readme_signals {
        mega_regex.push_str(&format!("({})|", &reg));
    }
    mega_regex.pop();
    debug!("Mega regex: {}", mega_regex);

    Ok(Regex::new(&mega_regex)?)
}

fn process_repo(config: &PanslopConfig, path: &PathBuf) -> color_eyre::Result<()> {
    let all_paths: Vec<PathBuf> = fs::read_dir(&path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let maybe_readme_path = all_paths.iter().find(|it| is_readme(it));
    let signals_regex = compile_readme_signal_mega_regex(&config)?;

    if let Some(readme_path) = &maybe_readme_path {
        debug!("README: {}", readme_path.to_string_lossy());

        let readme = fs::read_to_string(readme_path)?;
        let emdashes = EMDASH_REGEX.captures_iter(&readme).count();
        let emojis = Regex::new(&config.detect.emojis)?
            .captures_iter(&readme)
            .count();
        let length = readme.len();
        let signals = signals_regex.captures_iter(&readme).count();

        debug!("Emdashes: {}", emdashes);
        debug!("Emojis: {}", emojis);
        debug!("Readme length: {}", length);
        debug!("Readme signals (regex): {}", signals);
    } else {
        return Err(eyre!(
            "Could not find README for {}",
            path.to_string_lossy()
        ));
    }

    Ok(())
}

pub fn analyse(
    config: PathBuf,
    db: PathBuf,
    debug_override_repo_root: Option<PathBuf>,
) -> color_eyre::Result<()> {
    info!(
        "Start analysis process. Config: {}, DB: {}",
        config.to_string_lossy(),
        db.to_string_lossy()
    );

    let config_str = std::fs::read_to_string(config)?;
    let config_parsed: PanslopConfig = toml::from_str(&config_str)?;

    if let Some(debug_override) = debug_override_repo_root {
        if !debug_override.is_dir() {
            return Err(eyre!("Debug override path must be a dir"));
        }

        info!(
            "Use debug override repo: {}",
            debug_override.to_string_lossy()
        );
        process_repo(&config_parsed, &debug_override)?;
    }

    Ok(())
}
