systemctl restart vpp

vppctl set interface state GigabitEthernet0/3/0 up
vppctl set interface state GigabitEthernet0/4/0 up
vppctl set interface ip address GigabitEthernet0/3/0 192.168.179.2/24
vppctl set interface ip address GigabitEthernet0/4/0 10.0.1.1/24
vppctl ip route add 192.168.179.0/24 via 192.168.179.2 GigabitEthernet0/3/0
vppctl ip route add 10.0.1.0/24 via 10.0.1.1 GigabitEthernet0/4/0