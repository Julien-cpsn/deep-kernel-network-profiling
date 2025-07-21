virt-builder fedora-42 \
  --output images/fedora.qcow2 \
  --format qcow2 \
  --size 6G \
  --root-password password:fedora \
  --color \
  --install "bcc" \
  --mkdir "/mnt/shared" \
  --firstboot-command "mount -t 9p -o trans=virtio,version=9p2000.L shared_folder /mnt/shared"