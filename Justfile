[private]
default:
    @just --list

dev:
    direnv allow

fmt:
    just --unstable --fmt
    wuffsfmt -w ratmap-core-wuffs/src/*.wuffs

build: build-core


build-core:
    cat "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" > ./ratmap-core-wuffs/build/wuffs-base.c
    wuffs-c gen -genlinenum -package_name chunk_stream < ./ratmap-core-wuffs/src/chunk-stream.wuffs > ./ratmap-core-wuffs/build/chunk_stream.c
