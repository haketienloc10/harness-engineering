#!/usr/bin/env bash
set -Eeuo pipefail

cargo build --release -p harness-cli
install -m 755 target/release/harness-cli _harness/bin/harness-cli.next
mv _harness/bin/harness-cli.next _harness/bin/harness-cli
