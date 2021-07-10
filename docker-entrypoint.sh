#!/bin/bash
set -e

cargo run --features "${SWANLING_FEATURES}" --release --example "${SWANLING_EXAMPLE}" -- $@
