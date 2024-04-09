run:
  sudo capsh --keep=1 --user=$USER --inh=cap_net_admin --addamb=cap_net_admin -- -c 'RUST_BACKTRACE=1 cargo run --bin vmm -- --memory 512 --cpus 1 --kernel do-vmm/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin --network-host-ip 172.29.0.1 --network-host-netmask 255.255.0.0'

setup-agent:
  #!/usr/bin/env bash
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -w /volume \
    -t clux/muslrust \
    cargo build --release --package agent
  cp target/x86_64-unknown-linux-musl/release/agent do-vmm/rootfs/alpine-minirootfs/agent

build-kernel:
  pushd do-vmm/kernel/linux-cloud-hypervisor && \
    KCFLAGS="-Wa,-mx86-used-note=no" make bzImage -j `nproc`

cleanup:
  ps aux | grep "just run" | awk '{print $2}' | head -n 1 | xargs kill -9

mount:
  sudo mount -t proc /proc do-vmm/rootfs/alpine-minirootfs/proc
  sudo mount -t sysfs /sys do-vmm/rootfs/alpine-minirootfs/sys
  sudo mount --bind /dev do-vmm/rootfs/alpine-minirootfs/dev
  sudo mount --bind /run do-vmm/rootfs/alpine-minirootfs/run

chroot: mount
  sudo chroot do-vmm/rootfs/alpine-minirootfs /bin/sh

unmount:
  sudo umount do-vmm/rootfs/alpine-minirootfs/proc
  sudo umount do-vmm/rootfs/alpine-minirootfs/sys
  sudo umount do-vmm/rootfs/alpine-minirootfs/dev
  sudo umount do-vmm/rootfs/alpine-minirootfs/run
