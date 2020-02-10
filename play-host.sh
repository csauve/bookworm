#!/bin/sh
RUST_BACKTRACE=1 cargo run host -t 5000 \
  -s "http://127.0.0.1:8080" \
  -s "http://127.0.0.1:8080" \
  -s "http://127.0.0.1:8080" \
  -s "http://127.0.0.1:8080" \
  -p
