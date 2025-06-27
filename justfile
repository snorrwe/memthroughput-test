_default:
    @just --list

TEST := "memcpy"

run n='20' size='4294967296':
    #!/usr/bin/env bash

    cargo build --release
    target/release/memthroughput-test --repetitions={{ n }} {{ TEST }} --size {{ size }} -t 1
    target/release/memthroughput-test --repetitions={{ n }} {{ TEST }} --size {{ size }} -t $(nproc)
