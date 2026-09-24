#!/usr/bin/env bash
set -euo pipefail

# Usage: bash fuzz/run.sh [seconds-per-target] [seed]
seconds="${1:-20}"
seed="${2:-20260924}"
if [[ $# -gt 2 || ! "$seconds" =~ ^[1-9][0-9]{0,3}$ ]] || (( seconds > 3600 )); then
  echo 'Expected 1..3600 seconds per target and an optional numeric seed.' >&2
  exit 2
fi
if [[ ! "$seed" =~ ^[1-9][0-9]{0,8}$ ]]; then
  echo 'Expected a seed between 1 and 999999999.' >&2
  exit 2
fi

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
toolchain="${FUZZ_TOOLCHAIN:-nightly-2026-09-24}"
output="${FUZZ_OUTPUT_DIR:-$repo_root/fuzz/results}"
mkdir -p "$output"
output="$(cd -- "$output" && pwd)"

rustc "+$toolchain" --version --verbose > "$output/toolchain.txt"
cargo "+$toolchain" fuzz --version > "$output/cargo-fuzz.txt"
git rev-parse HEAD > "$output/source-commit.txt"
git diff --binary HEAD -- .gitattributes Cargo.toml src tests/support fuzz > "$output/source.patch"
printf 'seconds_per_target=%s\nseed=%s\nmax_len=4096\ntimeout=5\nrss_limit_mb=1024\nmalloc_limit_mb=16\n' \
  "$seconds" "$seed" > "$output/budget.txt"
cp fuzz/Cargo.lock "$output/Cargo.lock"

# cargo-fuzz has no --locked flag. Fetch against the committed lock first,
# build/run offline, and reject any lock drift before fuzzing or success.
cargo "+$toolchain" fetch --manifest-path fuzz/Cargo.toml --locked
CARGO_NET_OFFLINE=true cargo "+$toolchain" fuzz build 2>&1 | tee "$output/build.log"
cmp fuzz/Cargo.lock "$output/Cargo.lock"

for target in json base64 list url scalars; do
  mkdir -p "$output/corpus/$target" "$output/artifacts/$target"
  CARGO_NET_OFFLINE=true cargo "+$toolchain" fuzz run "$target" \
    "$output/corpus/$target" "$repo_root/fuzz/corpus/$target" -- \
    "-max_total_time=$seconds" "-seed=$seed" -max_len=4096 -timeout=5 \
    -rss_limit_mb=1024 -malloc_limit_mb=16 \
    "-artifact_prefix=$output/artifacts/$target/" -print_final_stats=1 \
    2>&1 | tee "$output/$target.log"
  cmp fuzz/Cargo.lock "$output/Cargo.lock"
done
