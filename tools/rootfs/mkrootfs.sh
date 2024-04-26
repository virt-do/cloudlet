#!/usr/bin/bash
# test if its already in src folder => fs-gen available
# launch from src folder at the same level as tools
if [ -d src ]
then
    cd src
fi

if [ -d fs-gen ]
then
    cargo run --bin fs-gen -- $1 ./fs-gen/test -o ../tools/rootfs/initramfs.img
else
    echo "Module fs-gen not found"
fi
