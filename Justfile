[private]
default:
  @just --list

dev:
  direnv allow

fmt:
  wuffsfmt -w ratmap-core/src/*.wuffs

build:
  cat "$(dirname $(dirname $(which wuffs)))/include/wuffs-v0.4.c" > ./ratmap-core/build/wuffs-base.c
  wuffs-c gen -genlinenum -package_name chunk_stream < ./ratmap-core/src/chunk-stream.wuffs > ./ratmap-core/build/chunk_stream.c
