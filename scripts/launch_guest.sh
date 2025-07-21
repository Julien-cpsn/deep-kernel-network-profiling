qemu-system-x86_64 -smp 4 -m 4096 -drive file=images/$1.qcow2 \
  -netdev tap,id=u0,ifname=tap0,script=no,downscript=no \
  -device virtio-net-pci,netdev=u0 \
  -device e1000 \
  -virtfs local,path=./shared,mount_tag=shared_folder,id=shared_folder,security_model=mapped-xattr \
  -nographic
