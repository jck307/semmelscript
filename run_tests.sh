#!/bin/bash
echo building...
cargo build --release || exit
echo -e "\nrunning tests...\n"
for file in $(find tests -maxdepth 1 -type f); do
    echo -e "\033[1m$file\033[0m"
    ./target/debug/semmel $file
    echo
done
echo "done."
