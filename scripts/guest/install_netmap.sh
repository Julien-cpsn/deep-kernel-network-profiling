cd /netmap/LINUX
make
make install
rmmod e1000
insmod ./netmap.ko
insmod ./e1000/e1000.ko

lb -i enp0s3 -i enp0s4