#!/usr/bin/bash
set -eoux pipefail
cd ~/services/panslopticon
./target/release/panslopticon update config.toml &> /tmp/job-update.log
