modprobe vfio-pci
dpdk-devbind.py --bind=e1000 0000:00:03.0
dpdk-devbind.py --bind=e1000 0000:00:04.0