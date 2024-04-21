setup:
  pushd tools/rootfs && \
    ./mkrootfs.sh

  pushd tools/kernel && \
    ./mkkernel.sh

run:
  sudo capsh --keep=1 --user=$USER --inh=cap_net_admin --addamb=cap_net_admin -- -c 'RUST_BACKTRACE=1 cargo run --bin vmm -- --memory 512 --cpus 1 --initramfs=/tools/rootfs/initramfs.img --kernel tools/kernel/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin --network-host-ip 172.29.0.1 --initramfs=/opt/repositories/virt-do/copies/cloudlet/tools/rootfs/initramfs.img --network-host-netmask 255.255.0.0'

build-kernel:
  pushd tools/kernel/linux-cloud-hypervisor && \
    KCFLAGS="-Wa,-mx86-used-note=no" make bzImage -j `nproc`
