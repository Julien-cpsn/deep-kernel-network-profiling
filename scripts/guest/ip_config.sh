ip link set enp0s3 up
ip address add 192.168.179.2/24 dev enp0s3

ip link set enp0s4 up
ip address add 10.0.1.1/24 dev enp0s4

echo 1 > /proc/sys/net/ipv4/ip_forward