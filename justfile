_default:
    @just --list

run n='20' size='4294967296':
    #!/usr/bin/env bash

    cargo build --release
    target/release/memthroughput-test --repetitions={{ n }} memcpy --size {{ size }} -t 1
    target/release/memthroughput-test --repetitions={{ n }} memcpy --size {{ size }} -t $(nproc)
