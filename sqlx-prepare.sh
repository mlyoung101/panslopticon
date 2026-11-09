#!/bin/bash
set -eoux pipefail

ruff check --select I --fix
ruff format

cargo sqlx prepare
git add .sqlx
