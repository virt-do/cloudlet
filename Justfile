set shell := ["/bin/bash", "-uc"]

setup:
  #!/bin/bash
  pushd tools/rootfs
  ./mkrootfs.sh
  popd

  pushd tools/kernel
  ./mkkernel.sh
  popd

run:
  #!/bin/bash
  CARGO_PATH=$(which cargo)
  sudo -E capsh --keep=1 --user=$USER --inh=cap_net_admin --addamb=cap_net_admin -- -c \
    'RUST_BACKTRACE=1 '$CARGO_PATH' run --bin vmm -- --memory 512 --cpus 1 \
    --kernel tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin \
    --network-host-ip 172.29.0.1 --network-host-netmask 255.255.0.0 \
    --initramfs=tools/rootfs/initramfs.img'

build-kernel:
  #!/bin/bash
  pushd tools/kernel/linux-cloud-hypervisor && \
    KCFLAGS="-Wa,-mx86-used-note=no" make bzImage -j `nproc`

build-rootfs:
  #!/bin/bash
  pushd tools/rootfs && \
    ./mkrootfs.sh
