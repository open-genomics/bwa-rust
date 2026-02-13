#!/bin/bash
# 构建索引的包装脚本
# 用法: ./scripts/run_index.sh <reference.fa> [output_prefix]

set -euo pipefail

REFERENCE="${1:?Usage: $0 <reference.fa> [output_prefix]}"
OUTPUT="${2:-ref}"

cargo run --release -- index "$REFERENCE" -o "$OUTPUT"
