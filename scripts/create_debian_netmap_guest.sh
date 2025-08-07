virt-builder debian-12 \
  --output images/debian_netmap.qcow2 \
  --format qcow2 \
  --size 6G \
  --hostname debian \
  --root-password password:debian \
  --color \
  --append-line "/etc/apt/sources.list:deb https://cloudfront.debian.net/debian sid main" \
  --install "netperf,bpfcc-tools,libbpfcc,libbpfcc-dev,linux-headers-generic,linux-source,build-essential,linux-perf,git" \
  --run-command "cd /usr/src && tar xaf linux-source-6.12.tar.xz && cd /" \
  --run-command "git clone https://github.com/luigirizzo/netmap.git" \
  --run-command "cd /netmap/LINUX && sed -i 's/(c_);/(enum hrtimer_restart (*)(struct hrtimer *))(c_);/' /netmap/LINUX/bsd_glue.h" \
  --run-command "./configure --kernel-dir=/lib/modules/6.12.38+deb13-amd64/build --kernel-sources=/usr/src/linux-source-6.12 --no-drivers=virtio_net.c" \
  --upload "scripts/guest/install_netmap.sh:/root/install_netmap.sh" \
  --upload "scripts/guest/mount_shared_folder.sh:/root/mount_shared_folder.sh" \
  --upload "scripts/guest/ip_config.sh:/root/ip_config.sh" \
  --upload "scripts/guest/find_ebpf_function.sh:/root/find_ebpf_function.sh" \
  --upload "scripts/guest/find_lib_function.sh:/root/find_lib_function.sh" \
  --firstboot "/root/install_netmap.sh" \
  --firstboot-command "/root/mount_shared_folder.sh && /root/ip_config.sh"
#./configure --kernel-dir=/boot/config-$(uname -r) --kernel-sources=/usr/src/linux-headers-$(uname -r)/build

#/root/shared/aya-network-deep-profiling -vvv -t > /root/shared/output.txt & sleep 3 && /usr/local/bin/pkt-gen -i enp0s3 -f tx && pkill --signal SIGINT aya-network-dee