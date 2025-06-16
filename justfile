_default:
    @just --list

_run threads n size:
    #!/usr/bin/env bash
    echo running {{ n }} tests with buffer size $(units -t {{ size }}bytes GiB)GiB and {{ threads }} 'thread(s)'

    for i in $(seq 1 {{ n }}); do
        target/release/memthroughput-test memcpy --size {{ size }} -t {{ threads }}
    done

run n='20' size='4294967296':
    #!/usr/bin/env bash

    cargo build --release
    just _run 1 {{ n }} {{ size }}
    just _run $(nproc) {{ n }} {{ size }}
