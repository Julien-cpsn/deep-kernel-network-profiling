virt-builder debian-12 \
  --output images/debian_dpdk.qcow2 \
  --format qcow2 \
  --size 6G \
  --hostname debian \
  --root-password password:debian \
  --color \
  --append-line "/etc/apt/sources.list:deb https://cloudfront.debian.net/debian sid main" \
  --install "netperf,bpfcc-tools,libbpfcc,libbpfcc-dev,linux-headers-generic,linux-perf,build-essential,dpdk,dpdk-dev" \
  --upload "scripts/guest/mount_shared_folder.sh:/root/mount_shared_folder.sh" \
  --upload "scripts/guest/ip_config.sh:/root/ip_config.sh" \
  --upload "scripts/guest/dpdk_config.sh:/root/dpdk_config.sh" \
  --upload "scripts/guest/find_ebpf_function.sh:/root/find_ebpf_function.sh" \
  --upload "scripts/guest/find_lib_function.sh:/root/find_lib_function.sh" \
  --firstboot-command "/root/mount_shared_folder.sh & /root/dpdk_config.sh"

# https://access.redhat.com/solutions/36741
# https://doc.dpdk.org/guides-20.08/linux_gsg/nic_perf_intel_platform.html -> 10.1.3. Linux boot command line

# sudo sysctl -w vm.nr_hugepages=4096
# sudo sysctl -w vm.hugetlb_shm_group=36

# sysctl -w vm.nr_hugepages=1024