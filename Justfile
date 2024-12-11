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

build-core: clean-core
    mkdir -p ./ratmap-core-wuffs/gen

    # Run `wuffs-v0.4.c` through the preprocessor so that `makeheaders` can understand it.
    gcc -xc -E "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" -o ./ratmap-core-wuffs/gen/wuffs-base.c
    makeheaders ./ratmap-core-wuffs/gen/wuffs-base.c

    # Now overwrite with the actual c file
    cat "$WUFFS_INCLUDE_PATH/wuffs-v0.4.c" > ./ratmap-core-wuffs/gen/wuffs-base.c

    # Generate c code from our .wuffs files
    wuffs-c gen -genlinenum -package_name ratmap_chunk_stream_handshake ./ratmap-core-wuffs/src/handshake.wuffs > ./ratmap-core-wuffs/gen/handshake.c

    # Make headers from the generated c code
    makeheaders ./ratmap-core-wuffs/gen/handshake.c

clean-core:
    rm -rf ./ratmap-core-wuffs/gen
