#!/usr/bin/bash
# test if its already in src folder => fs-gen available
# launch from src folder at the same level as tools
if [ -d src ]
then
    cd src
fi

# augment the open file limit
ulimit -Sn 8192

if [ -d fs-gen ]
then
    cargo run --bin fs-gen -- $1 $2 -o $3 --no-compression
else
    echo "Module fs-gen not found"
fi