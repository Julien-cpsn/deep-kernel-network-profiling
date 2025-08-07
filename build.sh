#!/bin/sh
FEATURE_FLAGS=""

for var in "$@"; do
  FEATURE_FLAGS="${FEATURE_FLAGS}aya-network-deep-profiling/${var} "
done

cargo build --config 'target."cfg(all())".runner="sudo -E"' --features "${FEATURE_FLAGS}"
cp target/debug/aya-network-deep-profiling shared
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 shared/aya-network-deep-profiling