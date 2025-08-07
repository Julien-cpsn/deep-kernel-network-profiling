wget https://fast.dpdk.org/rel/dpdk-24.11.2.tar.xz
tar -xvf dpdk-24.11.2.tar.xz
cd dpdk-stable-24.11.2
meson setup -Dcpu_instruction_set=generic build
cd build
ninja
ninja install
ldconfig
echo "export PKG_CONFIG_PATH=/usr/local/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH" >> ~/.bashrc
echo "export LD_LIBRARY_PATH=/usr/local/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH" >> ~/.bashrc
modprobe vfio
modprobe vfio-pci
/usr/local/bin/dpdk-devbind.py --bind=vfio-pci 0000:01:00.0