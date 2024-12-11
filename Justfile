[private]
default:
    @just --list

dev:
    direnv allow

fmt:
    just --unstable --fmt
    wuffsfmt -w ratmap-core-wuffs/src/*.wuffs

build: build-core

clean: clean-core

clean-core:
    rm -r ./ratmap-core-wuffs/build

build-core: clean-core
    mkdir ./ratmap-core-wuffs/build
    cat "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" > ./ratmap-core-wuffs/build/wuffs-base.c
    wuffs-c gen -genlinenum -package_name ratmap_chunk_stream_handshake \
        < ./ratmap-core-wuffs/src/handshake.wuffs \
        > ./ratmap-core-wuffs/build/handshake.c
