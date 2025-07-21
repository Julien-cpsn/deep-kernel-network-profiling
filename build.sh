cargo build --config 'target."cfg(all())".runner="sudo -E"'
cp target/debug/aya-network-deep-profiling shared
patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 shared/aya-network-deep-profiling