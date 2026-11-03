#!/usr/bin/bash
set -eoux pipefail
cd ~/services/panslopticon
./target/release/panslopticon analyse config.toml &> /tmp/job-analyse.log
