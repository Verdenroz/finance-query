#!/usr/bin/env bash
#
# Run the iai-callgrind instruction-count regression gate in a vanilla-glibc
# container.
#
# Why a container: iai-callgrind runs benchmarks under valgrind. On hosts whose
# glibc is compiled for x86-64-v4 (CachyOS/Arch with AVX-512, etc.), glibc's
# own startup code uses AVX-512 instructions that valgrind 3.25 cannot decode,
# so *every* glibc-linked binary dies with SIGILL before main(). Debian/Ubuntu
# glibc (what CI uses) has no such instructions, so we run the gate there.
#
# Usage:
#   scripts/bench-regression.sh                 # run the gate
#   scripts/bench-regression.sh --save-baseline base   # save a named baseline
#   scripts/bench-regression.sh --baseline base        # compare against it
#
# Any extra arguments are forwarded to `cargo bench --bench regression`.
set -euo pipefail

cd "$(dirname "$0")/.."

IMAGE="${BENCH_IMAGE:-rust:1-bookworm}"
RUNNER_VERSION="${IAI_RUNNER_VERSION:-0.16.1}"
TARGET_DIR="target/bench-regression"

mkdir -p "$TARGET_DIR"

exec docker run --rm \
    --security-opt seccomp=unconfined \
    -v "$PWD":/work -w /work \
    -v fq_bench_cargo_registry:/usr/local/cargo/registry \
    -v fq_bench_cargo_git:/usr/local/cargo/git \
    -e CARGO_TARGET_DIR="/work/${TARGET_DIR}" \
    -e IAI_RUNNER_VERSION="$RUNNER_VERSION" \
    "$IMAGE" bash -c '
        set -euo pipefail
        apt-get update -qq && apt-get install -y -qq valgrind >/dev/null
        cargo install iai-callgrind-runner --version "$IAI_RUNNER_VERSION" --quiet 2>/dev/null || true
        cargo bench --bench regression --features finance-query/risk -- "$@"
    ' bash "$@"
