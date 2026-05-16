#!/bin/bash
echo building...
cargo build || exit
echo -e "\nrunning tests...\n"
for file in tests/*; do
    echo -e "\033[1m$file\033[0m"
    ./target/debug/semmel $file
    echo
done
echo "done."
