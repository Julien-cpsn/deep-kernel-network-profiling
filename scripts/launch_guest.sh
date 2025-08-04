sudo qemu-system-x86_64 -M q35 \
  -cpu qemu64,+ssse3,+sse4.1,+sse4.2 -smp 4 \
  -m 4096 \
  -drive file=images/$1.qcow2 \
  -accel kvm,kernel-irqchip=split \
  -netdev tap,id=u0,ifname=tap0,script=no,downscript=no,vhost=on \
  -device intel-iommu,intremap=on,device-iotlb=on \
  -device ioh3420,id=pcie.1,chassis=1 \
  -device virtio-net-pci,bus=pcie.1,netdev=u0,csum=off,gso=off,guest_tso4=off,guest_tso6=off,guest_ecn=off \
  -device e1000 \
  -virtfs local,path=./shared,mount_tag=shared_folder,id=shared_folder,security_model=mapped-xattr \
  -nographic
# -cpu host,pmu=on
