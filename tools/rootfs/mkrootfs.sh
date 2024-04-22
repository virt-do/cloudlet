#!/usr/bin/bash

if [ ! -d alpine-minirootfs ]
then
    curl -O https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz

    mkdir alpine-minirootfs
    tar xf alpine-minirootfs-3.14.2-x86_64.tar.gz -C alpine-minirootfs
fi

pushd alpine-minirootfs
cat > init <<EOF
#! /bin/sh
#
# /init executable file in the initramfs
#
mount -t devtmpfs dev /dev
mount -t proc proc /proc
mount -t sysfs sysfs /sys

ip link set up dev lo

slattach -L /dev/ttyS1&

while ! ifconfig sl0 &> /dev/null; do
    sleep 1
done

ifconfig sl0 172.30.0.11 netmask 255.255.0.0 up

exec /sbin/getty -n -l /bin/sh 115200 /dev/console
poweroff -f
EOF

chmod +x init

find . -print0 |
    cpio --null --create --verbose --owner root:root --format=newc |
    xz -9 --format=lzma  > ../initramfs.img

popd
