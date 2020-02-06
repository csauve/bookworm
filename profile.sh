#!/bin/sh
./build.sh
perf record --call-graph dwarf -- ./target/release/bookworm benchmark
perf report
