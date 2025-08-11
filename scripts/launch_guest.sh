qemu-system-x86_64 -M q35 \
  -cpu qemu64,+ssse3,+sse4.1,+sse4.2 \
  -smp 4 \
  -m 4096M \
  -enable-kvm \
  -drive file=images/"$1".qcow2 \
  -virtfs local,path=./shared,mount_tag=shared_folder,id=shared_folder,security_model=mapped-xattr \
  -netdev tap,id=u0,ifname=tap0,script=no,downscript=no \
  -device e1000,netdev=u0 \
  -device e1000 \
  -nographic

#  -mem-prealloc \
#  -mem-path /dev/hugepages/libvirt/qemu \

# sudo sysctl -w vm.nr_hugepages=4096
# sudo sysctl -w vm.hugetlb_shm_group=36
# sudo ip route add 10.0.1.0/24 via 192.168.179.2