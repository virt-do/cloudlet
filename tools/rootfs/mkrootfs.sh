#!/usr/bin/bash

set -e

if [ ! -d alpine-minirootfs ]
then
    curl -O https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz

    mkdir alpine-minirootfs
    tar xf alpine-minirootfs-3.14.2-x86_64.tar.gz -C alpine-minirootfs
fi


pushd alpine-minirootfs
mkdir -p etc/cloudlet/agent
cp ../../../target/x86_64-unknown-linux-musl/release/agent agent
cp ../config.toml etc/cloudlet/agent/config.toml

cat > init <<EOF
#! /bin/sh
#
# /init executable file in the initramfs
#
mount -t devtmpfs dev /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys

ip link set up dev lo

ifconfig eth0 172.29.0.2 netmask 255.255.0.0 up

/agent

reboot
EOF

chmod +x init

find . -print0 |
    cpio --null --create --verbose --owner root:root --format=newc |
    xz -9 --format=lzma  > ../initramfs.img

popd
