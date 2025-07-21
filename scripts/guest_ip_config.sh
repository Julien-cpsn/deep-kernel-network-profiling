ip link set ens3 up
ip address add 192.168.179.2/24 dev ens3

ip link set ens4 up
ip address add 10.0.1.1/24 dev ens4

echo 1 > /proc/sys/net/ipv4/ip_forward