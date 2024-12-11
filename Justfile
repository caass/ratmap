[private]
default:
    @just --list

dev:
    direnv allow

fmt:
    just --unstable --fmt
    wuffsfmt -w rtmp-wuffs/src/*.wuffs

build: build-core


build-core:
    cat "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" > ./rtmp-wuffs/build/wuffs-base.c
    wuffs-c gen -genlinenum -package_name chunk_stream < ./rtmp-wuffs/src/chunk-stream.wuffs > ./rtmp-wuffs/build/chunk_stream.c
