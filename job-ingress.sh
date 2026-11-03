#!/usr/bin/bash
set -eoux pipefail
cd ~/services/panslopticon
./target/release/panslopticon gh-ingress config.toml &> /tmp/job-gh-ingress.log

# we're not really seeing any new ham anymore so ignore it
# ./target/release/panslopticon ham-ingress config.toml &> /tmp/job-ham.log
