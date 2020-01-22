#!/bin/sh
RUSTFLAGS="--emit=asm" cargo build --release
perf record --call-graph dwarf -- ./target/release/bookworm benchmark
perf report
