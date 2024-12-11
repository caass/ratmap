[private]
default:
    @just --list

dev:
    direnv allow

fmt:
    just --unstable --fmt
    wuffsfmt -w ratmap-core/src/*.wuffs

build: build-core


build-core:
    cat "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" > ./ratmap-core/build/wuffs-base.c
    wuffs-c gen -genlinenum -package_name chunk_stream < ./ratmap-core/src/chunk-stream.wuffs > ./ratmap-core/build/chunk_stream.c
