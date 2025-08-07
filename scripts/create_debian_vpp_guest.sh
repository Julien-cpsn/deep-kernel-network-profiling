virt-builder debian-12 \
  --output images/debian_vpp.qcow2 \
  --format qcow2 \
  --size 6G \
  --hostname debian \
  --root-password password:debian \
  --color \
  --append-line "/etc/apt/sources.list:deb https://cloudfront.debian.net/debian sid main" \
  --install "curl,gnupg,gnupg2" \
  --run-command "curl -fsSL https://packagecloud.io/fdio/2506/gpgkey | gpg --dearmor > /etc/apt/keyrings/fdio_2506-archive-keyring.gpg" \
  --append-line "/etc/apt/sources.list:deb [signed-by=/etc/apt/keyrings/fdio_2506-archive-keyring.gpg] https://packagecloud.io/fdio/2506/debian/ bookworm main" \
  --append-line "/etc/apt/sources.list:deb-src [signed-by=/etc/apt/keyrings/fdio_2506-archive-keyring.gpg] https://packagecloud.io/fdio/2506/debian/ bookworm main" \
  --install "netperf,bpfcc-tools,libbpfcc,libbpfcc-dev,linux-headers-generic,linux-perf,build-essential,dpdk,dpdk-dev,vpp,vpp-plugin-core,vpp-plugin-dpdk" \
  --run-command "usermod -a -G vpp root & newgrp vpp" \
  --upload "scripts/guest/mount_shared_folder.sh:/root/mount_shared_folder.sh" \
  --upload "scripts/guest/ip_config.sh:/root/ip_config.sh" \
  --upload "scripts/guest/dpdk_config.sh:/root/dpdk_config.sh" \
  --upload "scripts/guest/vpp_config.sh:/root/vpp_config.sh" \
  --upload "scripts/guest/find_ebpf_function.sh:/root/find_ebpf_function.sh" \
  --upload "scripts/guest/find_lib_function.sh:/root/find_lib_function.sh" \
  --firstboot-command "/root/mount_shared_folder.sh & /root/dpdk_config.sh & /root/vpp_config.sh"

# sudo sysctl -w vm.nr_hugepages=4096
# sudo sysctl -w vm.hugetlb_shm_group=36

# sysctl -w vm.nr_hugepages=1024