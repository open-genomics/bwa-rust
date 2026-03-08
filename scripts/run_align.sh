#!/bin/bash
# 对齐 reads 的包装脚本
# 用法: ./scripts/run_align.sh <index.fm> <reads.fq> [output.sam]

set -euo pipefail

INDEX="${1:?Usage: $0 <index.fm> <reads.fq> [output.sam]}"
READS="${2:?Usage: $0 <index.fm> <reads.fq> [output.sam]}"
OUTPUT="${3:-}"

if [ -n "$OUTPUT" ]; then
    cargo run --release -- align -i "$INDEX" "$READS" -o "$OUTPUT"
else
    cargo run --release -- align -i "$INDEX" "$READS"
fi
