#!/usr/bin/env bash
# 无需安装即可运行 cargo wrevive：从仓库根目录执行
# 用法: ./cargo-wrevive.sh build [OPTIONS]
# 例:   ./cargo-wrevive.sh build -p wrevive-example-contract
set -e
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"
exec cargo run -p cargo-wrevive --release -- "$@"
