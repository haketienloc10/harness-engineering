#!/usr/bin/env bash
set -Eeuo pipefail

cargo clean
cargo build --release -p harness-cli
cp target/release/harness-cli _harness/bin/harness-cli
chmod 755 _harness/bin/harness-cli
