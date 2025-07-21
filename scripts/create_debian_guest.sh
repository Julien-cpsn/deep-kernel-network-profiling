virt-builder debian-12 \
  --output images/debian.qcow2 \
  --format qcow2 \
  --size 6G \
  --hostname debian \
  --root-password password:debian \
  --color \
  --append-line "/etc/apt/sources.list:deb https://cloudfront.debian.net/debian sid main" \
  --install "bpfcc-tools,libbpfcc,libbpfcc-dev,linux-headers-generic,linux-perf" \
  --upload "scripts/guest_mount_shared_folder.sh:/root/mount_shared_folder.sh" \
  --upload "scripts/guest_ip_config.sh:/root/ip_config.sh" \
  --upload "scripts/find_ebpf_function.sh:/root/find_ebpf_function.sh" \
  --firstboot-command "./root/guest_mount_shared_folder.sh && ./root/ip_config.sh"

#  --append-line "/etc/apt/sources.list:deb https://cloudfront.debian.net/debian sid main" \
#  --install "bpfcc-tools,libbpfcc,libbpfcc-dev,linux-headers-generic" \
