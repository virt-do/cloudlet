setup:
  pushd tools/rootfs && \
    ./mkrootfs.sh

  pushd tools/kernel && \
    ./mkkernel.sh

  pushd tools/kernel/linux-cloud-hypervisor && \
    make menuconfig

run:
  sudo capsh --keep=1 --user=$USER --inh=cap_net_admin --addamb=cap_net_admin -- -c 'RUST_BACKTRACE=1 cargo run --bin vmm -- --memory 512 --cpus 1 --kernel tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin --network-host-ip 172.29.0.1 --network-host-netmask 255.255.0.0'

setup-agent:
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -w /volume \
    -t clux/muslrust \
    cargo build --release --package agent
  cp target/x86_64-unknown-linux-musl/release/agent tools/rootfs/alpine-minirootfs/agent

build-kernel:
  pushd tools/kernel/linux-cloud-hypervisor && \
    KCFLAGS="-Wa,-mx86-used-note=no" make bzImage -j `nproc`

cleanup:
  ps aux | grep "just run" | awk '{print $2}' | head -n 1 | xargs kill -9

mount:
  sudo mount -t proc /proc tools/rootfs/alpine-minirootfs/proc
  sudo mount -t sysfs /sys tools/rootfs/alpine-minirootfs/sys
  sudo mount --bind /dev tools/rootfs/alpine-minirootfs/dev
  sudo mount --bind /run tools/rootfs/alpine-minirootfs/run

chroot: mount
  sudo chroot tools/rootfs/alpine-minirootfs /bin/sh

unmount:
  sudo umount tools/rootfs/alpine-minirootfs/proc
  sudo umount tools/rootfs/alpine-minirootfs/sys
  sudo umount tools/rootfs/alpine-minirootfs/dev
  sudo umount tools/rootfs/alpine-minirootfs/run
