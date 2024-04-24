set shell := ["/bin/bash", "-uc"]

setup:
  #!/bin/bash
  set -e
  just build-kernel
  just build-rootfs

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
  pushd tools/kernel
  ./mkkernel.sh
  popd

build-agent args = "":
  #!/bin/bash
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -w /volume \
    -t clux/muslrust \
    cargo build --release --bin agent {{args}}

build-rootfs mode = "dev":
  #!/bin/bash
  set -e
  if [ "{{mode}}" = "dev" ]; then
    echo "Building rootfs in debug mode"
    just build-agent "--features debug-agent"
  else
    echo "Building rootfs in release mode"
    just build-agent
  fi
  pushd tools/rootfs
  ./mkrootfs.sh
  popd
